//! Esoterica: A modular, high-performance Rust framework for generating
//! constructed languages (conlangs).
//!
//! This library provides the core generation engine. For CLI usage, enable
//! the `cli` feature. For WebAssembly, use the `wasm` feature.

pub mod archetypes;
pub mod phonology;
pub mod morphology;
pub mod lexicon;
pub mod lexicon_structs;
pub mod sound_change;
pub mod syntax;
pub mod semantic_drift;
pub mod orthography;
pub mod collaborative;

// ── Feature-Gated Modules ────────────────────────────────────────────────

#[cfg(feature = "cli")]
pub mod args;
#[cfg(feature = "cli")]
pub mod atproto;
#[cfg(feature = "cli")]
pub mod tui;

#[cfg(feature = "wasm")]
pub mod wasm;
