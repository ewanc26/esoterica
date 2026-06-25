use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, value_delimiter = ',', help = "Phonology archetypes")]
    pub phonology: Vec<String>,

    #[arg(short = 'c', long, value_delimiter = ',', help = "Sound change rules")]
    pub sound_change: Vec<String>,

    #[arg(short, long, value_delimiter = ',', help = "Morphology rules")]
    pub morphology: Vec<String>,

    #[arg(short = 'x', long, help = "Syntax order (SVO, SOV, VSO, VOS, OVS, OSV)")]
    pub syntax: Option<String>,

    #[arg(short, long, help = "Output path for the generated lexicon")]
    pub output: Option<PathBuf>,

    #[arg(short = 'y', long, help = "Number of syllables per word (default: 2)")]
    pub syllables: Option<usize>,

    #[arg(short = 'n', long, help = "Number of entries in the lexicon (default: 100)")]
    pub lexicon_size: Option<usize>,

    #[arg(long, help = "Enable interactive TUI mode")]
    pub interactive: bool,

    #[arg(long, help = "Title for ATProto publication")]
    pub publish_title: Option<String>,

    #[arg(long, help = "URI of the target ATProto publication")]
    pub publication_uri: Option<String>,

    #[arg(long, help = "Number of semantic drift time-steps to simulate")]
    pub drift_steps: Option<usize>,

    #[arg(long, help = "Probability of drift per word per step (default: 0.15)")]
    pub drift_rate: Option<f64>,

    #[arg(long, help = "Generate a procedural orthography for the language")]
    pub generate_orthography: bool,
}

impl Args {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
