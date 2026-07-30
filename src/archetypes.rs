//! Linguistic archetype definitions and TOML-backed registries.
//! These types define the shape of every configurable component and are
//! deserialised from the data/ directory at compile time via include_str!.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// Phoneme inventory and phonotactic constraints for a language.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Phonology {
    pub vowels: Vec<String>,
    pub consonants: Vec<String>,
    /// Phonotactic template: C=consonant, V=vowel (e.g. "CVC", "CCCVCCCC")
    pub syllable_structure: String,
    /// Number of lexical tones, if tonal
    pub tones: Option<u8>,
    /// Whether vowels harmonise with the first vowel's front/back class
    pub vowel_harmony: Option<bool>,
}

/// A single sound-change rule in legacy TOML format.
/// Parser-based rules (e.g. "p > b / V_V") are handled by FormalRule in sound_change.rs.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SoundChange {
    pub pattern: String,
    pub replacement: String,
    /// Context constraint: "word_initial", "word_final", or None for unconditional
    pub context: Option<String>,
}

/// Morphological profile: affix rules and optional noun classes.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Morphology {
    pub rules: Vec<MorphRule>,
    pub noun_classes: Option<Vec<String>>,
}

/// Affix operations applied in order to a root word.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MorphRule {
    Suffix(String),
    Prefix(String),
    /// Inserted at the character midpoint of the stem
    Infix(String),
    /// A discontinuous affix wrapping the stem, as in German *ge-…-t*
    Circumfix(String, String),
    /// Doubles the stem with a hyphen separator
    Reduplication,
    /// Copies the first N characters of the stem as a reduplicant prefix
    PartialReduplication(usize),
}

/// The six attested orderings of subject, verb, and object.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum WordOrder {
    Svo,
    Sov,
    Vso,
    Vos,
    Ovs,
    Osv,
}

impl WordOrder {
    /// Every order, in descending order of cross-linguistic frequency.
    pub const ALL: [WordOrder; 6] = [
        WordOrder::Sov,
        WordOrder::Svo,
        WordOrder::Vso,
        WordOrder::Vos,
        WordOrder::Ovs,
        WordOrder::Osv,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            WordOrder::Svo => "SVO",
            WordOrder::Sov => "SOV",
            WordOrder::Vso => "VSO",
            WordOrder::Vos => "VOS",
            WordOrder::Ovs => "OVS",
            WordOrder::Osv => "OSV",
        }
    }

    /// Constituents in surface order, as `S`, `V`, and `O` markers.
    pub fn constituents(&self) -> [char; 3] {
        let s = self.as_str().as_bytes();
        [s[0] as char, s[1] as char, s[2] as char]
    }

    /// Whether the object precedes the verb, which correlates with
    /// postpositions and other head-final patterns.
    pub fn is_object_initial_of_verb(&self) -> bool {
        let s = self.as_str();
        s.find('O') < s.find('V')
    }
}

impl fmt::Display for WordOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Returned when a string does not name one of the six word orders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownWordOrder(pub String);

impl fmt::Display for UnknownWordOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown word order '{}' (expected one of SOV, SVO, VSO, VOS, OVS, OSV)",
            self.0
        )
    }
}

impl std::error::Error for UnknownWordOrder {}

impl FromStr for WordOrder {
    type Err = UnknownWordOrder;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "SVO" => Ok(WordOrder::Svo),
            "SOV" => Ok(WordOrder::Sov),
            "VSO" => Ok(WordOrder::Vso),
            "VOS" => Ok(WordOrder::Vos),
            "OVS" => Ok(WordOrder::Ovs),
            "OSV" => Ok(WordOrder::Osv),
            _ => Err(UnknownWordOrder(s.to_string())),
        }
    }
}

/// Whether adpositions precede or follow their complement.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Adposition {
    Preposition,
    Postposition,
}

/// Whether attributive adjectives precede or follow the noun.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdjectiveOrder {
    Prenominal,
    Postnominal,
}

/// Word-order configuration and case system.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Syntax {
    /// One of: SVO, SOV, VSO, VOS, OVS, OSV
    pub word_order: String,
    /// Case labels; the first two are used as subject and object markers.
    pub cases: Option<Vec<String>>,
    /// Adposition placement. Defaults to the head-direction implied by the
    /// word order when unset.
    #[serde(default)]
    pub adposition: Option<Adposition>,
    /// Attributive adjective placement, defaulted the same way.
    #[serde(default)]
    pub adjective_order: Option<AdjectiveOrder>,
}

impl Syntax {
    /// Parse `word_order` into the typed enum.
    pub fn parsed_word_order(&self) -> Result<WordOrder, UnknownWordOrder> {
        self.word_order.parse()
    }
}

// ── TOML Registry Loaders ────────────────────────────────────────────────

/// A registry file failed to parse.
#[derive(Debug)]
pub struct RegistryError {
    pub file: &'static str,
    pub source: toml::de::Error,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse {}: {}", self.file, self.source)
    }
}

impl std::error::Error for RegistryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

fn parse_registry<T: serde::de::DeserializeOwned>(
    file: &'static str,
    contents: &str,
) -> Result<T, RegistryError> {
    toml::from_str(contents).map_err(|source| RegistryError { file, source })
}

/// Load the phonology catalogue from data/phonologies.toml at compile time.
pub fn try_get_phonology_registry() -> Result<HashMap<String, Phonology>, RegistryError> {
    parse_registry("phonologies.toml", include_str!("../data/phonologies.toml"))
}

/// Load the sound change catalogue from data/sound_changes.toml.
pub fn try_get_sound_change_registry() -> Result<HashMap<String, Vec<SoundChange>>, RegistryError> {
    parse_registry("sound_changes.toml", include_str!("../data/sound_changes.toml"))
}

/// Load the morphology catalogue from data/morphologies.toml.
pub fn try_get_morphology_registry() -> Result<HashMap<String, Morphology>, RegistryError> {
    parse_registry("morphologies.toml", include_str!("../data/morphologies.toml"))
}

/// Load the syntax catalogue from data/syntaxes.toml.
pub fn try_get_syntax_registry() -> Result<HashMap<String, Syntax>, RegistryError> {
    parse_registry("syntaxes.toml", include_str!("../data/syntaxes.toml"))
}

/// Load the phonology catalogue, panicking on a malformed data file.
pub fn get_phonology_registry() -> HashMap<String, Phonology> {
    try_get_phonology_registry().expect("Failed to parse phonologies.toml")
}

/// Load the sound change catalogue, panicking on a malformed data file.
pub fn get_sound_change_registry() -> HashMap<String, Vec<SoundChange>> {
    try_get_sound_change_registry().expect("Failed to parse sound_changes.toml")
}

/// Load the morphology catalogue, panicking on a malformed data file.
pub fn get_morphology_registry() -> HashMap<String, Morphology> {
    try_get_morphology_registry().expect("Failed to parse morphologies.toml")
}

/// Load the syntax catalogue, panicking on a malformed data file.
pub fn get_syntax_registry() -> HashMap<String, Syntax> {
    try_get_syntax_registry().expect("Failed to parse syntaxes.toml")
}

// ── Registry Bundle ──────────────────────────────────────────────────────

/// All four archetype catalogues, loaded together.
pub struct Registries {
    pub phonologies: HashMap<String, Phonology>,
    pub morphologies: HashMap<String, Morphology>,
    pub syntaxes: HashMap<String, Syntax>,
    pub sound_changes: HashMap<String, Vec<SoundChange>>,
}

impl Registries {
    /// Load every catalogue, reporting the first malformed file.
    pub fn load() -> Result<Self, RegistryError> {
        Ok(Self {
            phonologies: try_get_phonology_registry()?,
            morphologies: try_get_morphology_registry()?,
            syntaxes: try_get_syntax_registry()?,
            sound_changes: try_get_sound_change_registry()?,
        })
    }

    /// Look up a phonology, listing the alternatives when the key is unknown.
    pub fn phonology(&self, key: &str) -> Result<Phonology, UnknownKey> {
        lookup(&self.phonologies, key, "phonology").cloned()
    }

    /// Look up a morphology, listing the alternatives when the key is unknown.
    pub fn morphology(&self, key: &str) -> Result<Morphology, UnknownKey> {
        lookup(&self.morphologies, key, "morphology").cloned()
    }

    /// Look up a syntax preset, listing the alternatives when the key is unknown.
    pub fn syntax(&self, key: &str) -> Result<Syntax, UnknownKey> {
        lookup(&self.syntaxes, key, "syntax").cloned()
    }

    /// Look up a sound-change set, listing the alternatives when the key is unknown.
    pub fn sound_change(&self, key: &str) -> Result<Vec<SoundChange>, UnknownKey> {
        lookup(&self.sound_changes, key, "sound change").cloned()
    }

    /// Concatenate several sound-change sets in the given order.
    ///
    /// Unlike silently skipping unknown keys, this reports the first one that
    /// does not exist so a typo cannot quietly disable a rule set.
    pub fn merged_sound_changes(&self, keys: &[String]) -> Result<Vec<SoundChange>, UnknownKey> {
        let mut merged = Vec::new();
        for key in keys {
            merged.extend(self.sound_change(key)?);
        }
        Ok(merged)
    }
}

/// A registry lookup missed, with the available keys for the error message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownKey {
    pub kind: &'static str,
    pub key: String,
    pub available: Vec<String>,
}

impl fmt::Display for UnknownKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown {} '{}'. Available: {}",
            self.kind,
            self.key,
            self.available.join(", ")
        )
    }
}

impl std::error::Error for UnknownKey {}

fn lookup<'a, T>(
    registry: &'a HashMap<String, T>,
    key: &str,
    kind: &'static str,
) -> Result<&'a T, UnknownKey> {
    registry.get(key).ok_or_else(|| {
        let mut available: Vec<String> = registry.keys().cloned().collect();
        available.sort();
        UnknownKey { kind, key: key.to_string(), available }
    })
}

/// Registry keys in a stable, sorted order — useful for CLI listings and for
/// anything that must not depend on `HashMap` iteration order.
pub fn sorted_keys<T>(registry: &HashMap<String, T>) -> Vec<&str> {
    let mut keys: Vec<&str> = registry.keys().map(String::as_str).collect();
    keys.sort_unstable();
    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_registries_parse() {
        assert!(Registries::load().is_ok());
    }

    #[test]
    fn registries_are_non_empty() {
        let r = Registries::load().unwrap();
        assert!(!r.phonologies.is_empty());
        assert!(!r.morphologies.is_empty());
        assert!(!r.syntaxes.is_empty());
        assert!(!r.sound_changes.is_empty());
    }

    #[test]
    fn every_syntax_preset_names_a_real_word_order() {
        let r = Registries::load().unwrap();
        for (name, syntax) in &r.syntaxes {
            assert!(
                syntax.parsed_word_order().is_ok(),
                "syntax preset '{}' has invalid word_order '{}'",
                name,
                syntax.word_order
            );
        }
    }

    #[test]
    fn all_six_word_orders_have_a_preset() {
        let r = Registries::load().unwrap();
        let present: std::collections::HashSet<WordOrder> = r
            .syntaxes
            .values()
            .filter_map(|s| s.parsed_word_order().ok())
            .collect();
        for order in WordOrder::ALL {
            assert!(present.contains(&order), "no syntax preset uses {}", order);
        }
    }

    #[test]
    fn word_order_round_trips_through_strings() {
        for order in WordOrder::ALL {
            assert_eq!(order.as_str().parse::<WordOrder>().unwrap(), order);
        }
    }

    #[test]
    fn word_order_parsing_is_case_insensitive() {
        assert_eq!("sov".parse::<WordOrder>().unwrap(), WordOrder::Sov);
        assert_eq!("Vso".parse::<WordOrder>().unwrap(), WordOrder::Vso);
    }

    #[test]
    fn unknown_word_order_is_reported() {
        let err = "SVX".parse::<WordOrder>().unwrap_err();
        assert_eq!(err, UnknownWordOrder("SVX".to_string()));
        assert!(err.to_string().contains("SVX"));
    }

    #[test]
    fn object_verb_ordering_is_derived_from_the_order() {
        assert!(WordOrder::Sov.is_object_initial_of_verb());
        assert!(WordOrder::Osv.is_object_initial_of_verb());
        assert!(!WordOrder::Svo.is_object_initial_of_verb());
        assert!(!WordOrder::Vos.is_object_initial_of_verb());
    }

    #[test]
    fn unknown_keys_list_the_alternatives() {
        let r = Registries::load().unwrap();
        let err = r.phonology("not_a_real_key").unwrap_err();
        assert_eq!(err.kind, "phonology");
        assert!(!err.available.is_empty());
        assert!(err.to_string().contains("Available:"));
    }

    #[test]
    fn merging_sound_changes_rejects_unknown_keys() {
        let r = Registries::load().unwrap();
        let err = r
            .merged_sound_changes(&["lenition".to_string(), "nope".to_string()])
            .unwrap_err();
        assert_eq!(err.key, "nope");
    }

    #[test]
    fn merging_sound_changes_concatenates_in_order() {
        let r = Registries::load().unwrap();
        let lenition = r.sound_change("lenition").unwrap();
        let rhotacism = r.sound_change("rhotacism").unwrap();
        let merged = r
            .merged_sound_changes(&["lenition".to_string(), "rhotacism".to_string()])
            .unwrap();
        assert_eq!(merged.len(), lenition.len() + rhotacism.len());
        assert_eq!(merged[0].pattern, lenition[0].pattern);
    }

    #[test]
    fn none_is_an_empty_sound_change_set() {
        let r = Registries::load().unwrap();
        assert!(r.sound_change("none").unwrap().is_empty());
    }

    #[test]
    fn sorted_keys_are_stable() {
        let r = Registries::load().unwrap();
        let first = sorted_keys(&r.phonologies);
        let second = sorted_keys(&r.phonologies);
        assert_eq!(first, second);
        assert!(first.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn morph_rules_deserialise_from_toml() {
        #[derive(Deserialize)]
        struct Wrapper {
            rules: Vec<MorphRule>,
        }
        let toml_src = r#"
            rules = [
                { suffix = "-en" },
                { prefix = "ti-" },
                { infix = "-ka-" },
                { circumfix = ["ge", "t"] },
                { partial_reduplication = 2 },
                "reduplication",
            ]
        "#;
        let parsed: Wrapper = toml::from_str(toml_src).unwrap();
        assert_eq!(
            parsed.rules,
            vec![
                MorphRule::Suffix("-en".to_string()),
                MorphRule::Prefix("ti-".to_string()),
                MorphRule::Infix("-ka-".to_string()),
                MorphRule::Circumfix("ge".to_string(), "t".to_string()),
                MorphRule::PartialReduplication(2),
                MorphRule::Reduplication,
            ]
        );
    }

    #[test]
    fn syntax_typological_fields_default_to_none() {
        let syntax: Syntax = toml::from_str("word_order = \"SVO\"").unwrap();
        assert_eq!(syntax.adposition, None);
        assert_eq!(syntax.adjective_order, None);
        assert_eq!(syntax.parsed_word_order().unwrap(), WordOrder::Svo);
    }
}
