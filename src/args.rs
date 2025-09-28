use clap::Parser;
use std::sync::LazyLock;

pub static ARGS: LazyLock<Args> = LazyLock::new(Args::parse);

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// The packages to check
    pub packages: Vec<String>,

    /// Do not write changes
    #[arg(short, long)]
    pub pretend: bool,

    /// Ensure every package is checked
    #[arg(short, long)]
    pub guarantee: bool,
}
