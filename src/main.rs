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
    #[arg(short, long, value_delimiter = ',')]
    phonology: Vec<String>,

    #[arg(short, long, value_delimiter = ',')]
    sound_change: Vec<String>,

    #[arg(short, long, value_delimiter = ',')]
    morphology: Vec<String>,

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
    
    // Merge Phonologies
    let mut merged_phono = archetypes::Phonology {
        vowels: Vec::new(),
        consonants: Vec::new(),
        syllable_structure: "CVC".to_string(),
        tones: None,
        vowel_harmony: None,
    };
    for key in &args.phonology {
        if let Some(p) = phono_reg.get(key) {
            merged_phono.vowels.extend(p.vowels.clone());
            merged_phono.consonants.extend(p.consonants.clone());
            merged_phono.vowels.sort();
            merged_phono.vowels.dedup();
            merged_phono.consonants.sort();
            merged_phono.consonants.dedup();
        }
    }

    // Merge Morphologies
    let mut merged_morph = archetypes::Morphology { rules: Vec::new() };
    for key in &args.morphology {
        if let Some(m) = morph_reg.get(key) {
            merged_morph.rules.extend(m.rules.clone());
        }
    }

    // Sound Changes
    let mut merged_sc = Vec::new();
    for key in &args.sound_change {
        if let Some(sc) = sc_reg.get(key) {
            merged_sc.extend(sc.clone());
        }
    }

    let syntax = syntax_reg.get(&args.syntax).ok_or_else(|| anyhow!("Unknown syntax: {}", args.syntax))?.clone();

    let mut generator = lexicon::LexiconGenerator::new(merged_phono, merged_morph, merged_sc);
    generator.generate_core_lexicon(100);
    generator.save_to_file(args.output.to_str().context("Invalid output path")?)?;

    // Syntax Usage
    let syntax_engine = syntax::SyntaxEngine::new(syntax);
    let sentence = syntax_engine.generate_sentence(&["word1".to_string(), "word2".to_string(), "word3".to_string()]);

    println!("Generated language with Phono: {:?}, Morph: {:?}, Syntax: {}", args.phonology, args.morphology, args.syntax);
    println!("Example sentence: {}", sentence);
    println!("Lexicon saved to: {:?}", args.output);
    Ok(())
}
