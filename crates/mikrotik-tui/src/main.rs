//! MikroTik TUI — keyboard-first RouterOS control deck (Rust).

use clap::Parser;
use mtui_core::DefaultTheme;

#[derive(Debug, Parser)]
#[command(name = "mikrotik-tui", version, about = "RouterOS control deck")]
struct Args {
    /// Disable the alternate screen buffer.
    #[arg(long)]
    no_alt_screen: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let _ = mtui_config::init_file_logging();
    tracing::info!(theme = DefaultTheme::ID, "starting mikrotik-tui (Rust)");
    mtui_app::run(!args.no_alt_screen)
}
