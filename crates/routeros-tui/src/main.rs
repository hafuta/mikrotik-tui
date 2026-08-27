//! `RouterOS` TUI: keyboard-first control deck (Rust).

use clap::Parser;
use mtui_core::DefaultTheme;

#[derive(Debug, Parser)]
#[command(name = "routeros-tui", version, about = "RouterOS control deck")]
struct Args {
    /// Disable the alternate screen buffer.
    #[arg(long)]
    no_alt_screen: bool,
    /// Open the fixture demo profile (no router required).
    #[arg(long)]
    demo: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let _ = mtui_config::init_file_logging();
    tracing::info!(theme = DefaultTheme::ID, "starting routeros-tui (Rust)");
    mtui_app::run(!args.no_alt_screen, args.demo)
}
