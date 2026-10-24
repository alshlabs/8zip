use std::fs;
use std::io::Read;
use std::io::Seek;
use std::io::{self};
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use zip::ZipArchive;

use crate::input_target::InputTarget;

pub fn decompress_zip(
  target: InputTarget,
  output: &Path,
  strip_components: Option<usize>,
  path: Option<PathBuf>,
) -> Result<()> {
  // Create output directory if it doesn't exist
  fs::create_dir_all(output).context(format!("Failed to create output directory: {:?}", output))?;

  // Create the appropriate reader based on input type
  match target {
    InputTarget::Path(target_path) => {
      let resolved = crate::platform::cmd_utils::resolve_path(target_path.to_str().unwrap())?;

      // Validate that the archive exists
      if !resolved.exists() {
        anyhow::bail!("Archive does not exist: {:?}", resolved);
      }

      let file =
        fs::File::open(&resolved).context(format!("Failed to open archive: {:?}", resolved))?;

      extract_zip(file, output, strip_count(strip_components), path)
    }
    InputTarget::Stdin(_) => {
      // Zip requires random access, so we need to read all of stdin into memory
      let mut buffer = Vec::new();
      io::stdin()
        .read_to_end(&mut buffer)
        .context("Failed to read zip archive from stdin")?;

      let cursor = io::Cursor::new(buffer);
      extract_zip(cursor, output, strip_count(strip_components), path)
    }
  }
}

fn strip_count(strip_components: Option<usize>) -> usize {
  strip_components.unwrap_or(0)
}

fn extract_zip<R: Read + Seek>(
  reader: R,
  output: &Path,
  strip_count: usize,
  path: Option<PathBuf>,
) -> Result<()> {
  // Create zip archive reader
  let mut archive = ZipArchive::new(reader).context("Failed to read zip archive")?;

  // Extract each file in the archive
  for i in 0..archive.len() {
    let mut file = archive
      .by_index(i)
      .context(format!("Failed to access zip entry {}", i))?;

    let original_path = match file.enclosed_name() {
      Some(path) => path.to_path_buf(),
      None => continue, // Skip entries with unsafe paths
    };

    // Strip components first
    let stripped_path = if strip_count > 0 {
      let components: Vec<_> = original_path.components().collect();

      // Skip entries that don't have enough components
      if components.len() <= strip_count {
        continue;
      }

      // Build new path without the first n components
      let new_path: PathBuf = components[strip_count..].iter().collect();

      // Skip if the resulting path is empty
      if new_path.as_os_str().is_empty() {
        continue;
      }

      new_path
    } else {
      original_path
    };

    // If a specific path filter is provided, check if this entry matches
    if let Some(ref filter_path) = path {
      // Check if the stripped path starts with the filter path
      if !stripped_path.starts_with(filter_path) {
        continue;
      }
    }

    let outpath = output.join(&stripped_path);

    if file.name()?.ends_with('/') {
      // Create directory
      fs::create_dir_all(&outpath).context(format!("Failed to create directory: {:?}", outpath))?;
    } else {
      // Create parent directories if needed
      if let Some(parent) = outpath.parent() {
        fs::create_dir_all(parent)
          .context(format!("Failed to create parent directory: {:?}", parent))?;
      }

      // Extract file
      let mut outfile =
        fs::File::create(&outpath).context(format!("Failed to create file: {:?}", outpath))?;
      io::copy(&mut file, &mut outfile)
        .context(format!("Failed to extract file: {:?}", outpath))?;
    }

    // Set permissions on Unix systems
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      if let Some(mode) = file.unix_mode() {
        fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))
          .context(format!("Failed to set permissions: {:?}", outpath))?;
      }
    }
  }

  Ok(())
}
