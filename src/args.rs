//! CLI argument definitions via clap derive.
//! Defines the full surface area of the command-line interface.

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Phonology archetype key from data/phonologies.toml
    #[arg(short, long, value_delimiter = ',')]
    pub phonology: Vec<String>,

    /// Sound change rule set key(s) from data/sound_changes.toml (comma-separated)
    #[arg(short = 'c', long, value_delimiter = ',')]
    pub sound_change: Vec<String>,

    /// Morphology archetype key from data/morphologies.toml
    #[arg(short, long, value_delimiter = ',')]
    pub morphology: Vec<String>,

    /// Word order (SVO, SOV, VSO, VOS, OVS, OSV)
    #[arg(short = 'x', long)]
    pub syntax: Option<String>,

    /// Output path for the generated lexicon
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Syllables per word (default: 2)
    #[arg(short = 'y', long)]
    pub syllables: Option<usize>,

    /// Lexicon entry count (default: 100)
    #[arg(short = 'n', long)]
    pub lexicon_size: Option<usize>,

    /// Launch the interactive TUI instead of CLI generation
    #[arg(long)]
    pub interactive: bool,

    /// Title for ATProto dictionary publication
    #[arg(long)]
    pub publish_title: Option<String>,

    /// ATRecord URI of the target publication
    #[arg(long)]
    pub publication_uri: Option<String>,

    /// Number of semantic drift time-steps
    #[arg(long)]
    pub drift_steps: Option<usize>,

    /// Per-word-per-step drift probability (default: 0.15)
    #[arg(long)]
    pub drift_rate: Option<f64>,

    /// Generate a procedural orthography/script
    #[arg(long)]
    pub generate_orthography: bool,
}

impl Args {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
