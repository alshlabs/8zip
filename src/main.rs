mod compress_tar_gz;
mod compress_tar_xz;
mod compress_zip;
mod decompress_tar_gz;
mod decompress_tar_xz;
mod decompress_zip;
mod input_target;
mod platform;
mod upgrade;

use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;

use crate::input_target::InputTarget;

#[derive(Debug, Parser)]
struct Command {
  #[command(subcommand)]
  command: CommandType,
}

#[derive(Debug, Subcommand)]
enum CommandType {
  Compress {
    target: PathBuf,
    #[arg(long="cwd", value_parser = platform::cmd_utils::resolve_path, default_value = ".")]
    cwd: PathBuf,
    #[arg(long="output", value_parser = platform::cmd_utils::resolve_path)]
    output: PathBuf,
  },
  CompressTarGz {
    target: PathBuf,
    #[arg(long="cwd", value_parser = platform::cmd_utils::resolve_path, default_value = ".")]
    cwd: PathBuf,
    #[arg(long="output", value_parser = platform::cmd_utils::resolve_path)]
    output: PathBuf,
  },
  CompressTarXz {
    target: PathBuf,
    #[arg(long="cwd", value_parser = platform::cmd_utils::resolve_path, default_value = ".")]
    cwd: PathBuf,
    #[arg(long="output", value_parser = platform::cmd_utils::resolve_path)]
    output: PathBuf,
  },
  CompressZip {
    target: PathBuf,
    #[arg(long="cwd", value_parser = platform::cmd_utils::resolve_path, default_value = ".")]
    cwd: PathBuf,
    #[arg(long="output", value_parser = platform::cmd_utils::resolve_path)]
    output: PathBuf,
  },
  Extract {
    target: Option<PathBuf>,
    #[arg(long = "strip-components")]
    strip_components: Option<usize>,
    #[arg(long="output", value_parser = platform::cmd_utils::resolve_path)]
    output: PathBuf,
    #[arg(long = "path")]
    path: Option<PathBuf>,
  },
  ExtractTarGz {
    target: Option<PathBuf>,
    #[arg(long = "strip-components")]
    strip_components: Option<usize>,
    #[arg(long="output", value_parser = platform::cmd_utils::resolve_path)]
    output: PathBuf,
    #[arg(long = "path")]
    path: Option<PathBuf>,
  },
  ExtractTarXz {
    target: Option<PathBuf>,
    #[arg(long = "strip-components")]
    strip_components: Option<usize>,
    #[arg(long="output", value_parser = platform::cmd_utils::resolve_path)]
    output: PathBuf,
    #[arg(long = "path")]
    path: Option<PathBuf>,
  },
  ExtractZip {
    target: Option<PathBuf>,
    #[arg(long = "strip-components")]
    strip_components: Option<usize>,
    #[arg(long="output", value_parser = platform::cmd_utils::resolve_path)]
    output: PathBuf,
    #[arg(long = "path")]
    path: Option<PathBuf>,
  },
  /// Update the binary to the latest release
  Upgrade(upgrade::UpgradeCommand),
}

fn main() -> anyhow::Result<()> {
  let args = Command::parse();
  //   dbg!(&args);

  match args.command {
    CommandType::Compress {
      target,
      cwd,
      output,
    } => match output.extension().map(|v| v.to_str()) {
      Some(Some("gz")) => compress_tar_gz::compress_tar_gz(&target, &cwd, &output),
      Some(Some("xz")) => compress_tar_xz::compress_tar_xz(&target, &cwd, &output),
      Some(Some("zip")) => compress_zip::compress_zip(&target, &cwd, &output),
      _ => Err(anyhow::anyhow!(
        "Unable to detect extension for {:?}",
        output.extension()
      )),
    },
    CommandType::CompressTarGz {
      target,
      cwd,
      output,
    } => compress_tar_gz::compress_tar_gz(&target, &cwd, &output),
    CommandType::CompressTarXz {
      target,
      cwd,
      output,
    } => compress_tar_xz::compress_tar_xz(&target, &cwd, &output),
    CommandType::CompressZip {
      target,
      cwd,
      output,
    } => compress_zip::compress_zip(&target, &cwd, &output),
    CommandType::ExtractTarGz {
      target,
      strip_components,
      output,
      path,
    } => decompress_tar_gz::decompress_tar_gz(
      InputTarget::detect(target)?,
      &output,
      strip_components,
      path,
    ),
    CommandType::ExtractTarXz {
      target,
      strip_components,
      output,
      path,
    } => decompress_tar_xz::decompress_tar_xz(
      InputTarget::detect(target)?,
      &output,
      strip_components,
      path,
    ),
    CommandType::ExtractZip {
      target,
      strip_components,
      output,
      path,
    } => decompress_zip::decompress_zip(
      InputTarget::detect(target)?,
      &output,
      strip_components,
      path,
    ),
    CommandType::Extract {
      target,
      strip_components,
      output,
      path,
    } => match target.as_ref().map(|v| v.extension().map(|v| v.to_str())) {
      Some(Some(Some("gz"))) => decompress_tar_gz::decompress_tar_gz(
        InputTarget::detect(target)?,
        &output,
        strip_components,
        path,
      ),
      Some(Some(Some("xz"))) => decompress_tar_xz::decompress_tar_xz(
        InputTarget::detect(target)?,
        &output,
        strip_components,
        path,
      ),
      Some(Some(Some("zip"))) => decompress_zip::decompress_zip(
        InputTarget::detect(target)?,
        &output,
        strip_components,
        path,
      ),
      _ => Err(anyhow::anyhow!("Unable to detect extension")),
    },
    CommandType::Upgrade(args) => upgrade::main(&args),
  }
}
