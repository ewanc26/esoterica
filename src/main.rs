pub mod archetypes;
pub mod phonology;
pub mod morphology;
pub mod lexicon;
pub mod sound_change;
pub mod syntax;
pub mod tui;
pub mod atproto;
pub mod args;

use args::Args;
use color_eyre::eyre::Result as EyreResult;

#[tokio::main]
async fn main() -> EyreResult<()> {
    // Install error handler
    color_eyre::install()?;

    let args = Args::parse_args();
    
    if args.interactive {
        tui::run_tui(args)?;
        return Ok(());
    }

    // ... (rest of main logic, wrapped in EyreResult)
    Ok(())
}
