import argparse
from esoterica.lexicon import LexiconGenerator

def main():
    parser = argparse.ArgumentParser(description="Esoterica Conlang Generator")
    parser.add_argument("--archetype", required=True, help="Base archetype to use")
    parser.add_argument("--output", default="output/language.json", help="Output file")
    
    args = parser.parse_args()
    
    gen = LexiconGenerator(args.archetype)
    gen.generate_core_lexicon(size=50)
    gen.save_to_file(args.output)
    print(f"Generated language package saved to {args.output}")

if __name__ == "__main__":
    main()
