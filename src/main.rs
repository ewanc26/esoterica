pub mod archetypes;
pub mod phonology;
pub mod morphology;
pub mod lexicon;
pub mod sound_change;
pub mod syntax;

use clap::Parser;
use std::path::PathBuf;
use anyhow::{Result, Context, anyhow};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    phonology: String,

    #[arg(short, long)]
    sound_change: String,

    #[arg(short, long)]
    morphology: String,

    #[arg(short, long)]
    syntax: String,

    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let phono_reg = archetypes::get_phonology_registry();
    let sc_reg = archetypes::get_sound_change_registry();
    let morph_reg = archetypes::get_morphology_registry();
    let syntax_reg = archetypes::get_syntax_registry();
    
    let phono = phono_reg.get(&args.phonology).ok_or_else(|| anyhow!("Unknown phonology: {}", args.phonology))?.clone();
    let sc = sc_reg.get(&args.sound_change).ok_or_else(|| anyhow!("Unknown sound change: {}", args.sound_change))?.clone();
    let morph = morph_reg.get(&args.morphology).ok_or_else(|| anyhow!("Unknown morphology: {}", args.morphology))?.clone();
    let syntax = syntax_reg.get(&args.syntax).ok_or_else(|| anyhow!("Unknown syntax: {}", args.syntax))?.clone();

    let mut generator = lexicon::LexiconGenerator::new(phono, morph, sc);
    generator.generate_core_lexicon(100);
    generator.save_to_file(args.output.to_str().context("Invalid output path")?)?;

    // Example Syntax Usage
    let syntax_engine = syntax::SyntaxEngine::new(syntax);
    let sentence = syntax_engine.generate_sentence(&["word1".to_string(), "word2".to_string(), "word3".to_string()]);

    println!("Generated language with Phono: {}, Morph: {}, Syntax: {}", args.phonology, args.morphology, args.syntax);
    println!("Example sentence: {}", sentence);
    println!("Lexicon saved to: {:?}", args.output);
    Ok(())
}
