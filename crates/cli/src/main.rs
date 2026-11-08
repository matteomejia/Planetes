use std::{
    io::{self, Write},
    path::PathBuf,
};

use clap::{Parser, ValueEnum};
use planetes_stores::{ScanOptions, scan};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Store {
    All,
    Steam,
    Epic,
}

/// Discover installed Steam and Epic titles locally and print a JSON report.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(long, value_enum, default_value_t = Store::All)]
    store: Store,
    /// Steam client directory (contains steamapps/libraryfolders.vdf).
    #[arg(long)]
    steam_root: Option<PathBuf>,
    /// Epic's directory of .item manifests.
    #[arg(long)]
    epic_manifests: Option<PathBuf>,
    /// Exit with status 2 if any scan warnings occur. The JSON report is still printed.
    #[arg(long)]
    strict: bool,
}

fn main() -> anyhow::Result<std::process::ExitCode> {
    let args = Args::parse();
    let report = scan(&ScanOptions {
        scan_steam: !matches!(args.store, Store::Epic),
        scan_epic: !matches!(args.store, Store::Steam),
        steam_root: args.steam_root,
        epic_manifests: args.epic_manifests,
    });
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    serde_json::to_writer_pretty(&mut stdout, &report)?;
    writeln!(stdout)?;
    stdout.flush()?;
    Ok(if args.strict && !report.warnings.is_empty() {
        std::process::ExitCode::from(2)
    } else {
        std::process::ExitCode::SUCCESS
    })
}
