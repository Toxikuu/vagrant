use color_eyre::config::HookBuilder;
use tracing::{debug, info};
use std::time::Instant;
use std::{env, fs};
use std::path::Path;

use color_eyre::Result;
use tracing_subscriber::fmt::time;
use tracing_subscriber::EnvFilter;
use self::args::ARGS;
use self::package::{bulk, Package};

mod utils;
mod package;
mod args;

fn main() -> color_eyre::Result<()> {
    let _ = fs::remove_dir_all(".vagrant-cache");
    let start_timestamp = Instant::now();

    HookBuilder::default()
        .display_env_section(true)
        .display_location_section(true)
        .add_default_filters()
        .install()?;

    log();

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
        fs::write("elapsed", &elapsed)?;
    }

    info!("Finished in {elapsed}");
    let _ = fs::remove_dir_all(".vagrant-cache");
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
