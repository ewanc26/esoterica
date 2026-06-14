# Esoterica Conlang Generator

Esoterica is a modular, high-performance Rust framework for generating constructed languages (conlangs). It allows users to create consistent languages by composing independent linguistic components: phonology, morphology, and syntax.

## Features

- **Component-based Generation**: Dynamically mix-and-match phonological, morphological, and syntactic profiles to create diverse languages.
- **Rule-based Synthesis**: 
    - **Phonology Engine**: Generates syllables based on custom phonotactic patterns (e.g., `C(C)V(C)`) with support for tone and vowel harmony.
    - **Morphology Engine**: Supports complex transformation rules including Suffix/Prefix/Infix insertion and Reduplication.
    - **Sound Change Engine**: Simulates diachronic phonological shifts for historical consistency.
- **CLI Interface**: Generate complex language packages directly from the command line.
- **Interactive TUI**: Real-time conlang generation and configuration.
- **ATProto Publication**: Publish generated lexicons directly to the ATProto network using `site.standard.document` schemas.

## Quick Start

1. **Build the project:**
   ```bash
   cargo build --release
   ```

2. **Run in Interactive TUI Mode:**
   ```bash
   cargo run --release -- --interactive
   ```
   *Navigate with `Tab`, input configurations, and press `Enter` to generate.*

3. **Generate via CLI:**
   ```bash
   cargo run --release -- --phonology uralic_finnic --morphology agglutinative --syntax svo --output my_language.json
   ```

## ATProto Publication

To publish your generated lexicon to ATProto, you need to use an **App Password** (not your main account password).

1. Get an App Password: https://bsky.app/settings/app-passwords
2. Configure the following environment variables:
   ```bash
   export ATPROTO_HANDLE="your-handle.bsky.social"
   export ATPROTO_PASSWORD="your-app-password"
   ```
3. Run the generator with the publication flag:
   ```bash
   cargo run --release -- \
     --phonology uralic_finnic \
     --morphology agglutinative \
     --syntax svo \
     --publish-title "My New Language" \
     --publication-uri "at://did:plc:.../site.standard.publication/..."
   ```

## Project Structure

```
esoterica/
├── src/
│   ├── main.rs            # CLI and orchestration logic
│   ├── archetypes.rs      # Registry of linguistic components
│   ├── phonology.rs       # Syllable and word generation engine
│   ├── morphology.rs      # Morphological transformation engine
│   ├── sound_change.rs    # Diachronic sound change simulation
│   ├── syntax.rs          # Word order management
│   ├── lexicon.rs         # Dictionary generation and management
│   ├── tui.rs             # Ratatui interactive interface
│   └── atproto.rs         # ATProto publishing logic
├── Cargo.toml             # Rust dependencies
└── README.md              # Project documentation
```

## Contributing

Contributions are welcome! Feel free to submit a pull request for new archetypes, morphological rules, or advanced features.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
