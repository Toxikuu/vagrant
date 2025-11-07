use color_eyre::config::HookBuilder;
use color_eyre::eyre::WrapErr;
use tracing::{debug, info};
use std::os::unix::fs::MetadataExt;
use std::sync::LazyLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{env, fs};
use std::path::{Path, PathBuf};

use color_eyre::Result;
use tracing_subscriber::fmt::time;
use tracing_subscriber::EnvFilter;
use self::args::ARGS;
use self::package::{bulk, Package};

mod utils;
mod package;
mod args;

/// Timeout for .vagrant-cache
const CACHE_TIMEOUT: Duration = Duration::from_secs(3600); // 1 hr

static VAGRANT_ROOT: LazyLock<PathBuf> = LazyLock::new(|| {
    env::current_dir().expect("Couldn't get working directory")
});

static SHLIB_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    VAGRANT_ROOT.join("sh/lib.env")
});

fn main() -> color_eyre::Result<()> {
    clean_cache()?;
    let start_timestamp = Instant::now();

    HookBuilder::default()
        .display_env_section(true)
        .display_location_section(true)
        .add_default_filters()
        .install()?;

    log();

    debug!("Determined Vagrant root to be {}", VAGRANT_ROOT.display());

    let packages = if ARGS.packages.is_empty() {
        bulk::find_all()?
    } else {
        ARGS.packages.iter().map(|s| Package::from_name(s.clone())).collect::<Result<Vec<_>>>()?
    };

    debug!("Detected packages: {packages:#?}");
    let map = bulk::fetch_all(&packages)?;

    let elapsed = humantime::format_duration(start_timestamp.elapsed()).to_string();

    if !ARGS.pretend {
        bulk::write_all(&map)?;
        increment_runcount()?;
        debug!("Incremented runcount");
        fs::write(".vagrant-cache/elapsed", &elapsed)?;
    }

    info!("Finished in {elapsed}");
    Ok(())
}

fn log() {
    let level = env::var("LOG_LEVEL").unwrap_or_else(|_| String::from("info"));
    let filter = EnvFilter::new(level);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_level(true)
        .with_target(true)
        .with_line_number(true)
        .with_timer(time::uptime())
        .with_writer(std::io::stdout)
        .compact()
        .init();
}

fn increment_runcount() -> Result<()> {
    let path = Path::new("runcount");
    let runcount = fs::read_to_string(path).ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0u64) + 1;
    fs::write(path, runcount.to_string())?;
    Ok(())
}

fn clean_cache() -> Result<()> {
    let cache_path = Path::new(".vagrant-cache");
    if let Ok(m) = cache_path.metadata() {
        #[allow(clippy::cast_sign_loss)]
        let mtime = Duration::from_secs(m.mtime() as u64);
        let now = SystemTime::now().duration_since(UNIX_EPOCH)
            .wrap_err("Time travel detected")?;

        if now - mtime > CACHE_TIMEOUT {
            debug!("Removing cache");
            fs::remove_dir_all(cache_path)
                .wrap_err("Failed to remove cache")?;
            fs::create_dir(cache_path)
                .wrap_err("Failed to create cache")?;
        }
    } else {
        fs::create_dir(cache_path)
            .wrap_err("Failed to create cache")?;
    }

    Ok(())
}
