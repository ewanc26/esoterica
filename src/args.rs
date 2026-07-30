//! CLI argument definitions via clap derive.
//! Defines the full surface area of the command-line interface.

use clap::{Parser, ValueEnum};
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

    /// Syntax preset key from data/syntaxes.toml
    #[arg(short = 'x', long)]
    pub syntax: Option<String>,

    /// Ad-hoc sound change in formal notation, e.g. "p > b / V_V" (repeatable)
    #[arg(long = "formal-rule", value_name = "RULE")]
    pub formal_rules: Vec<String>,

    /// Output path for the generated lexicon
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Syllables per word (default: 2)
    #[arg(short = 'y', long)]
    pub syllables: Option<usize>,

    /// Lexicon entry count (default: 100)
    #[arg(short = 'n', long)]
    pub lexicon_size: Option<usize>,

    /// Seed for reproducible generation; omit for a different language each run
    #[arg(short = 's', long)]
    pub seed: Option<u64>,

    /// List every available archetype key and exit
    #[arg(short = 'l', long)]
    pub list: bool,

    /// Print a sample of the generated lexicon to stdout
    #[arg(long, value_name = "COUNT", num_args = 0..=1, default_missing_value = "10")]
    pub preview: Option<usize>,

    /// Number of example sentences to generate from the lexicon
    #[arg(long, default_value_t = 1)]
    pub sentences: usize,

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

    /// Writing system to generate with --generate-orthography
    #[arg(long, value_enum, default_value_t = ScriptTypeArg::Alphabet)]
    pub script_type: ScriptTypeArg,

    /// Visual style of generated glyphs
    #[arg(long, value_enum, default_value_t = GlyphStyleArg::Angular)]
    pub glyph_style: GlyphStyleArg,
}

/// Writing system selector, mirroring `orthography::ScriptType`.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
#[value(rename_all = "kebab-case")]
pub enum ScriptTypeArg {
    Alphabet,
    Abjad,
    Abugida,
    Syllabary,
    Logography,
}

/// Glyph aesthetic selector, mirroring `orthography::GlyphStyle`.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
#[value(rename_all = "kebab-case")]
pub enum GlyphStyleArg {
    Angular,
    Curved,
    Minimal,
    Ornate,
}

impl From<ScriptTypeArg> for crate::orthography::ScriptType {
    fn from(value: ScriptTypeArg) -> Self {
        use crate::orthography::ScriptType;
        match value {
            ScriptTypeArg::Alphabet => ScriptType::Alphabet,
            ScriptTypeArg::Abjad => ScriptType::Abjad,
            ScriptTypeArg::Abugida => ScriptType::Abugida,
            ScriptTypeArg::Syllabary => ScriptType::Syllabary,
            ScriptTypeArg::Logography => ScriptType::Logography,
        }
    }
}

impl From<GlyphStyleArg> for crate::orthography::GlyphStyle {
    fn from(value: GlyphStyleArg) -> Self {
        use crate::orthography::GlyphStyle;
        match value {
            GlyphStyleArg::Angular => GlyphStyle::Angular,
            GlyphStyleArg::Curved => GlyphStyle::Curved,
            GlyphStyleArg::Minimal => GlyphStyle::Minimal,
            GlyphStyleArg::Ornate => GlyphStyle::Ornate,
        }
    }
}

impl Args {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Args::command().debug_assert();
    }

    fn parse(argv: &[&str]) -> Args {
        Args::try_parse_from(std::iter::once("esoterica").chain(argv.iter().copied())).unwrap()
    }

    #[test]
    fn defaults_are_empty() {
        let args = parse(&[]);
        assert!(args.phonology.is_empty());
        assert!(args.seed.is_none());
        assert!(!args.list);
        assert_eq!(args.sentences, 1);
        assert_eq!(args.script_type, ScriptTypeArg::Alphabet);
    }

    #[test]
    fn comma_separated_sound_changes_split() {
        let args = parse(&["--sound-change", "lenition,rhotacism"]);
        assert_eq!(args.sound_change, vec!["lenition", "rhotacism"]);
    }

    #[test]
    fn formal_rules_are_repeatable() {
        let args = parse(&["--formal-rule", "p > b / V_V", "--formal-rule", "k > h / _#"]);
        assert_eq!(args.formal_rules, vec!["p > b / V_V", "k > h / _#"]);
    }

    #[test]
    fn seed_is_parsed() {
        assert_eq!(parse(&["--seed", "42"]).seed, Some(42));
        assert_eq!(parse(&["-s", "7"]).seed, Some(7));
    }

    #[test]
    fn preview_defaults_when_given_no_value() {
        assert_eq!(parse(&["--preview"]).preview, Some(10));
        assert_eq!(parse(&["--preview", "3"]).preview, Some(3));
        assert_eq!(parse(&[]).preview, None);
    }

    #[test]
    fn script_and_style_enums_parse() {
        let args = parse(&["--script-type", "syllabary", "--glyph-style", "ornate"]);
        assert_eq!(args.script_type, ScriptTypeArg::Syllabary);
        assert_eq!(args.glyph_style, GlyphStyleArg::Ornate);
    }

    #[test]
    fn unknown_script_type_is_rejected() {
        assert!(Args::try_parse_from(["esoterica", "--script-type", "runes"]).is_err());
    }

    #[test]
    fn script_type_maps_to_the_engine_enum() {
        use crate::orthography::ScriptType;
        assert_eq!(ScriptType::from(ScriptTypeArg::Abugida), ScriptType::Abugida);
        assert_eq!(ScriptType::from(ScriptTypeArg::Logography), ScriptType::Logography);
    }

    #[test]
    fn glyph_style_maps_to_the_engine_enum() {
        use crate::orthography::GlyphStyle;
        assert_eq!(GlyphStyle::from(GlyphStyleArg::Curved), GlyphStyle::Curved);
        assert_eq!(GlyphStyle::from(GlyphStyleArg::Minimal), GlyphStyle::Minimal);
    }
}
