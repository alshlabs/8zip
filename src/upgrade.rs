use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use clap::Parser;
use self_update::ReleaseAsset;
use self_update::backends::github;

const BIN_NAME: &str = "8zip";
const TARGET_REPO: &str = "alshlabs/8zip";

#[derive(Debug, Parser)]
pub struct UpgradeCommand {
  /// Check whether a newer version is available without installing it
  #[clap(long)]
  pub check: bool,
}

pub fn main(args: &UpgradeCommand) -> anyhow::Result<()> {
  let current_version = env!("CARGO_PKG_VERSION");

  if args.check {
    return check(current_version);
  }

  println!("Starting Self Upgrade");

  match is_prerelease(current_version)? {
    true => println!("Already on the latest version ({current_version})"),
    false => match configure(current_version)?
      .progress_callback(progress())
      .build()?
      .update()
    {
      Ok(self_update::VersionStatus::Updated(_)) => println!("Upgrade complete"),
      Ok(_) => println!("Already on the latest version ({current_version})"),
      Err(err) => eprintln!("Upgrade failed: {err}"),
    },
  }

  Ok(())
}

fn check(current_version: &str) -> anyhow::Result<()> {
  if is_prerelease(current_version)? {
    println!("Already on the latest version ({current_version})");
    return Ok(());
  }

  let releases = configure(current_version)?.build()?.get_latest_release()?;
  let Some(release) = releases.latest() else {
    println!("Already on the latest version ({current_version})");
    return Ok(());
  };

  let remote_version = normalize_tag(release.version())?;
  match semver::Version::parse(current_version)? < remote_version {
    true => println!("Update available: {current_version} -> {remote_version}"),
    false => println!("Already on the latest version ({current_version})"),
  }

  Ok(())
}

fn configure(current_version: &str) -> anyhow::Result<github::UpdateBuilder> {
  let Some((owner, name)) = TARGET_REPO.split_once('/') else {
    anyhow::bail!("Invalid repo name");
  };

  let mut builder = github::Update::configure();
  builder
    .repo_owner(owner)
    .repo_name(name)
    .bin_name(BIN_NAME)
    .current_version(current_version)
    .asset_matcher(select_asset)
    .unattended();

  Ok(builder)
}

fn is_prerelease(version: &str) -> anyhow::Result<bool> {
  Ok(!semver::Version::parse(version)?.pre.is_empty())
}

fn normalize_tag(tag: &str) -> anyhow::Result<semver::Version> {
  let tag = tag.strip_prefix('v').unwrap_or(tag);
  Ok(semver::Version::parse(tag)?)
}

fn select_asset(assets: &[ReleaseAsset]) -> Option<ReleaseAsset> {
  let os = match std::env::consts::OS {
    "windows" => &["win32", "windows", "win"][..],
    "macos" => &["macos", "darwin"][..],
    "linux" => &["linux"][..],
    _ => return None,
  };

  let arch = match std::env::consts::ARCH {
    "x86_64" => &["amd64", "x86_64"][..],
    "aarch64" => &["arm64", "aarch64"][..],
    _ => return None,
  };

  assets
    .iter()
    .find(|asset| {
      let name = asset.name().to_ascii_lowercase();
      name.ends_with(".tar.gz")
        && os.iter().any(|s| name.contains(s))
        && arch.iter().any(|s| name.contains(s))
    })
    .cloned()
}

fn progress() -> impl Fn(u64, Option<u64>) + Send + Sync + 'static {
  let started = AtomicBool::new(false);
  let reported = AtomicU64::new(0);

  move |downloaded, total| {
    if !started.swap(true, Ordering::Relaxed) {
      println!("0%");
    }

    let Some(total) = total.filter(|total| *total > 0) else {
      return;
    };

    let step = (downloaded * 4 / total).min(4);
    if step > reported.load(Ordering::Relaxed) {
      reported.store(step, Ordering::Relaxed);
      println!("{}%", step * 25);
    }
  }
}
