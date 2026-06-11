# Esoterica Conlang Generator

Esoterica is a modular, Python-based framework for generating constructed languages (conlangs). It allows users to create consistent languages by selecting from various natural language families and esoteric archetypes.

## Features

- **Archetype-based Generation:** Select influences from natural language families (e.g., Indo-European, Turkic) or esoteric archetypes (e.g., Ithkuil-like, Toki Pona-like).
- **Phonology Engine:** Generates sounds and syllable structures specific to the chosen influence.
- **Morphology Engine:** Implements word-building rules based on linguistic typologies (agglutinative, root-and-pattern, etc.).
- **Lexicon Management:** Generates core vocabularies and exports them as structured JSON.
- **CLI Interface:** Easily generate language packages via the command line.

## Quick Start

1. **Install dependencies** (standard Python library used).
2. **Generate a language:**
   ```bash
   python3 cli.py --archetype turkic --output output/my_language.json
   ```

## Project Structure

```
esoterica/
├── archetypes.py    # Definitions of linguistic profiles
├── phonology.py     # Phoneme generation and phonotactics
├── morphology.py    # Word structure and grammar rules
├── lexicon.py       # Dictionary generation
├── cli.py           # Command-line interface
└── tests/           # Test suite
```

## Contributing

Contributions are welcome! Please feel free to submit a pull request for new archetypes, morphological rules, or features.
