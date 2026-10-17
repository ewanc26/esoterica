//! Linguistic archetype definitions and TOML-backed registries.
//! These types define the shape of every configurable component and are
//! deserialised from the data/ directory at compile time via include_str!.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Word orders recognised by the syntax engine.
pub const WORD_ORDERS: [&str; 6] = ["SVO", "SOV", "VSO", "VOS", "OVS", "OSV"];

/// Phoneme inventory and phonotactic constraints for a language.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Phonology {
    pub vowels: Vec<String>,
    pub consonants: Vec<String>,
    /// Phonotactic template: C=consonant, V=vowel, parentheses mark an optional
    /// segment group (e.g. "CV", "(C)V(C)", "CCCVCCCC").
    pub syllable_structure: String,
    /// Number of lexical tones, if tonal
    pub tones: Option<u8>,
    /// Whether vowels harmonise with the first vowel's front/back class
    pub vowel_harmony: Option<bool>,
}

impl Phonology {
    /// Check that the inventory and template can drive the generator without panicking.
    pub fn validate(&self) -> Result<(), String> {
        if self.vowels.is_empty() {
            return Err("vowel inventory is empty".into());
        }
        if self.syllable_structure.contains('C') && self.consonants.is_empty() {
            return Err("template uses C but consonant inventory is empty".into());
        }
        if self.vowels.iter().chain(self.consonants.iter()).any(|s| s.is_empty()) {
            return Err("inventory contains an empty segment".into());
        }
        // A consonant spelled only with vowel letters is almost always a glide written
        // with the wrong letter (e.g. "y" for /j/), and would be read as a vowel.
        if let Some(c) = self.consonants.iter().find(|s| s.chars().all(crate::sound_change::is_vowel)) {
            return Err(format!("consonant '{}' is spelled with vowel letters; use 'j' for the glide", c));
        }
        if let Some(t) = self.tones {
            if t > 9 {
                return Err(format!("{} tones is more than the 9 superscript tone marks supported", t));
            }
        }
        validate_template(&self.syllable_structure)
    }
}

/// Check a syllable template: only C, V, and balanced non-nested parentheses,
/// with at least one V so every syllable has a nucleus.
pub fn validate_template(template: &str) -> Result<(), String> {
    let mut depth = 0u8;
    for ch in template.chars() {
        match ch {
            'C' | 'V' => {}
            '(' if depth == 0 => depth = 1,
            ')' if depth == 1 => depth = 0,
            '(' | ')' => return Err(format!("unbalanced or nested parentheses in '{}'", template)),
            other => return Err(format!("unexpected '{}' in template '{}'", other, template)),
        }
    }
    if depth != 0 {
        return Err(format!("unclosed parenthesis in '{}'", template));
    }
    if !template.contains('V') {
        return Err(format!("template '{}' has no vowel slot", template));
    }
    Ok(())
}

/// A single sound-change rule in legacy TOML format.
/// Retained for the legacy engine; registry entries use formal notation.
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
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MorphRule {
    Suffix(String),
    Prefix(String),
    /// Inserted at the midpoint of the root
    Infix(String),
    /// Doubles the root with a hyphen separator
    Reduplication,
    /// Non-concatenative root-and-pattern morphology. `C` takes the next consonant
    /// of the root (cycling if the root runs out); every other character is literal.
    /// e.g. "CaCaC" interleaves a root with a vocalic pattern.
    Template(String),
}

/// Word-order configuration and case system.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Syntax {
    /// One of: SVO, SOV, VSO, VOS, OVS, OSV
    pub word_order: String,
    /// Case labels: first is the subject/agent case, second the object/patient case.
    pub cases: Option<Vec<String>>,
    /// Alignment of the case system, e.g. "nominative-accusative" or "ergative-absolutive".
    #[serde(default)]
    pub alignment: Option<String>,
    /// Adposition placement, e.g. "prepositional", "postpositional".
    #[serde(default)]
    pub adpositions: Option<String>,
}

impl Syntax {
    pub fn validate(&self) -> Result<(), String> {
        if !WORD_ORDERS.contains(&self.word_order.as_str()) {
            return Err(format!("unknown word order '{}'", self.word_order));
        }
        if let Some(cases) = &self.cases {
            if cases.len() > 2 {
                return Err("at most two case labels (subject, object) are used".into());
            }
        }
        Ok(())
    }
}

// ── TOML Registry Loaders ────────────────────────────────────────────────

/// Load the phonology catalogue at compile time: the curated inventories in
/// data/phonologies.toml, plus the generated Glottolog representatives in
/// data/phonologies_glottolog.toml. Generated keys end in a Glottolog code, so the
/// two sets never collide.
pub fn get_phonology_registry() -> HashMap<String, Phonology> {
    let mut map: HashMap<String, Phonology> = toml::from_str(include_str!("../data/phonologies.toml"))
        .expect("Failed to parse phonologies.toml");
    let generated: HashMap<String, Phonology> = toml::from_str(include_str!("../data/phonologies_glottolog.toml"))
        .expect("Failed to parse phonologies_glottolog.toml");
    for (key, value) in generated {
        let previous = map.insert(key.clone(), value);
        assert!(previous.is_none(), "duplicate phonology key '{}'", key);
    }
    map
}

/// Load the sound change catalogue from data/sound_changes.toml.
/// Each set is a list of formal rules (e.g. "p > b / V_V") applied in order.
pub fn get_sound_change_registry() -> HashMap<String, Vec<String>> {
    toml::from_str(include_str!("../data/sound_changes.toml")).expect("Failed to parse sound_changes.toml")
}

/// Load the morphology catalogue from data/morphologies.toml.
pub fn get_morphology_registry() -> HashMap<String, Morphology> {
    toml::from_str(include_str!("../data/morphologies.toml")).expect("Failed to parse morphologies.toml")
}

/// Load the syntax catalogue from data/syntaxes.toml.
pub fn get_syntax_registry() -> HashMap<String, Syntax> {
    toml::from_str(include_str!("../data/syntaxes.toml")).expect("Failed to parse syntaxes.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every entry in every registry must parse and pass its own validation,
    /// so a bad TOML edit fails here rather than panicking the CLI, TUI or WASM.
    #[test]
    fn all_registry_entries_are_valid() {
        if let Err(errors) = crate::pipeline::Registries::builtin().validate() {
            panic!("invalid registry entries:\n{}", errors.join("\n"));
        }
    }

    #[test]
    fn template_validation_rejects_bad_input() {
        assert!(validate_template("CV").is_ok());
        assert!(validate_template("(C)V(C)").is_ok());
        assert!(validate_template("CCCVCCCC").is_ok());
        assert!(validate_template("").is_err());
        assert!(validate_template("CC").is_err(), "no nucleus");
        assert!(validate_template("(CV").is_err());
        assert!(validate_template("C((V))").is_err());
        assert!(validate_template("CXV").is_err());
    }

    #[test]
    fn vowel_spelled_consonant_is_rejected() {
        let p = Phonology { vowels: vec!["a".into()], consonants: vec!["p".into(), "y".into()], syllable_structure: "CV".into(), tones: None, vowel_harmony: None };
        let err = p.validate().unwrap_err();
        assert!(err.contains("use 'j'"), "{}", err);
    }

    #[test]
    fn tone_count_above_nine_is_rejected() {
        let mut p = Phonology { vowels: vec!["a".into()], consonants: vec!["p".into()], syllable_structure: "CV".into(), tones: Some(9), vowel_harmony: None };
        assert!(p.validate().is_ok());
        p.tones = Some(10);
        assert!(p.validate().is_err());
    }

    #[test]
    fn empty_inventory_is_rejected() {
        let p = Phonology { vowels: vec![], consonants: vec!["p".into()], syllable_structure: "CV".into(), tones: None, vowel_harmony: None };
        assert!(p.validate().is_err());
    }

    #[test]
    fn unknown_word_order_is_rejected() {
        let s = Syntax { word_order: "XYZ".into(), cases: None, alignment: None, adpositions: None };
        assert!(s.validate().is_err());
    }

    /// Threshold for "most language families": each major family below (the largest
    /// recognised groups by speaker count, plus the main isolates and creoles) must have
    /// at least one inventory in the registry. Sub-branches are not required.
    #[test]
    fn major_families_are_represented() {
        let phono = get_phonology_registry();
        let major: [(&str, &[&str]); 39] = [
            ("Indo-European", &["eurasia_ie_germanic", "eurasia_ie_romance", "eurasia_ie_slavic", "eurasia_ie_indoaryan", "eurasia_ie_iranian"]),
            ("Sino-Tibetan", &["asia_sinotibetan_sinitic", "asia_sinotibetan_burmese", "asia_sinotibetan_tibetic"]),
            ("Afro-Asiatic", &["africa_afroasiatic_semitic", "africa_afroasiatic_berber", "africa_afroasiatic_cushitic", "africa_afroasiatic_chadic", "africa_afroasiatic_omotic"]),
            ("Niger-Congo", &["africa_nigercongo_bantu", "africa_nigercongo_yoruboid", "africa_nigercongo_mande", "africa_nigercongo_atlantic", "africa_nigercongo_gur", "africa_nigercongo_kwa", "africa_nigercongo_ubangian"]),
            ("Austronesian", &["oceania_austronesian", "oceania_austronesian_polynesian", "asia_austronesian_malayo", "asia_austronesian_philippine", "oceania_austronesian_malagasy"]),
            ("Dravidian", &["asia_dravidian"]),
            ("Turkic", &["eurasia_turkic"]),
            ("Austroasiatic", &["asia_austroasiatic_monkhmer"]),
            ("Tai-Kadai", &["asia_taikadai_tai"]),
            ("Japonic", &["asia_japonic"]),
            ("Koreanic", &["asia_koreanic"]),
            ("Uralic", &["uralic_finnic", "uralic_ugric", "eurasia_uralic_samoyedic"]),
            ("Mongolic", &["asia_mongolic"]),
            ("Tungusic", &["asia_tungusic"]),
            ("Kartvelian", &["caucasus_kartvelian"]),
            ("Nakh-Dagestanian", &["caucasus_nakh_dagestanian"]),
            ("Northwest Caucasian", &["caucasus_northwestcaucasian"]),
            ("Basque (isolate)", &["eurasia_isolate_basque"]),
            ("Nilo-Saharan", &["africa_nilosaharan_nilotic", "africa_nilosaharan_saharan"]),
            ("Khoisan", &["africa_khoisan_khoe", "africa_khoisan_tuu"]),
            ("Uto-Aztecan", &["americas_utoaztecan"]),
            ("Quechuan", &["americas_quechuan"]),
            ("Algic", &["americas_algic"]),
            ("Eskaleut", &["americas_eskaleut"]),
            ("Mayan", &["americas_mayan"]),
            ("Tupian", &["americas_tupian"]),
            ("Arawakan", &["americas_arawakan"]),
            ("Oto-Manguean", &["americas_otomanguean"]),
            ("Siouan", &["americas_siouan"]),
            ("Iroquoian", &["americas_iroquoian"]),
            ("Athabaskan", &["americas_athabaskan"]),
            ("Pama-Nyungan", &["oceania_pamanyungan"]),
            ("Trans-New Guinea", &["oceania_papuan_transnewguinea"]),
            ("Chukotko-Kamchatkan", &["eurasia_chukotko_kamchatkan"]),
            ("Yeniseian (isolate)", &["eurasia_isolate_yeniseian"]),
            ("Hmong-Mien", &["asia_hmongmien"]),
            ("Viet-Muong", &["asia_vietic"]),
            ("Creole", &["creole_tok_pisin", "creole_haitian"]),
            ("Constructed", &["constructed_toki_pona", "constructed_esperanto"]),
        ];
        let missing: Vec<&str> = major.iter()
            .filter(|(_, keys)| !keys.iter().any(|k| phono.contains_key(*k)))
            .map(|(family, _)| *family).collect();
        assert!(missing.is_empty(), "major families without an inventory: {:?}", missing);
    }

    /// The generated Glottolog set: sign languages are excluded by design, and known
    /// families are present by their Glottolog code.
    #[test]
    fn generated_catalogue_excludes_sign_languages_and_keeps_known_families() {
        let phono = get_phonology_registry();
        assert!(!phono.keys().any(|k| k.ends_with("_sign1238")), "sign languages must not be generated");
        let generated: Vec<&String> = phono.keys().filter(|k| k.len() > 9 && k.chars().rev().take(4).all(|c| c.is_ascii_digit())).collect();
        assert!(generated.len() >= 240, "expected at least 240 generated inventories, got {}", generated.len());
        for code in ["indo1319", "afro1255", "atla1278", "aust1307", "sino1245", "pama1250"] {
            assert!(phono.keys().any(|k| k.ends_with(&format!("_{}", code))), "missing family {}", code);
        }
    }

    /// Named coverage checklist for what the framework ships. Each family and
    /// typological type listed here must have at least one registry entry.
    #[test]
    fn coverage_checklist_is_present() {
        let phono = get_phonology_registry();
        let required_phonology = [
            // Indo-European
            "eurasia_ie_germanic", "eurasia_ie_romance", "eurasia_ie_slavic", "eurasia_ie_indoaryan",
            "eurasia_ie_celtic", "eurasia_ie_iranian",
            // Uralic, Turkic, Basque (isolate), Kartvelian, Northwest Caucasian
            "uralic_finnic", "uralic_ugric", "eurasia_turkic", "eurasia_isolate_basque",
            "caucasus_kartvelian", "caucasus_northwestcaucasian",
            // Afro-Asiatic, Niger-Congo, Nilo-Saharan, Khoisan
            "africa_afroasiatic_semitic", "africa_afroasiatic_berber", "africa_afroasiatic_cushitic",
            "africa_nigercongo_bantu", "africa_nigercongo_yoruboid", "africa_nigercongo_mande",
            "africa_nilosaharan_nilotic", "africa_khoisan_khoe",
            // Sino-Tibetan, Dravidian, Austroasiatic, Tai-Kadai, Japonic, Koreanic
            "asia_sinotibetan_sinitic", "asia_sinotibetan_burmese", "asia_dravidian",
            "asia_austroasiatic_monkhmer", "asia_taikadai_tai", "asia_japonic", "asia_koreanic",
            // Austronesian, Papuan, Pama-Nyungan
            "oceania_austronesian", "oceania_austronesian_polynesian", "oceania_papuan_transnewguinea",
            "oceania_pamanyungan",
            // Americas
            "americas_utoaztecan", "americas_quechuan", "americas_algic", "americas_salishan",
            "americas_eskaleut", "americas_mayan", "americas_arawakan",
            // Second batch: Mongolic, Tungusic, Tibetic, Hmong-Mien, Viet-Muong, Chadic,
            // Athabaskan, Oto-Manguean, Tupian, Siouan, Hellenic, Baltic, Armenian, Albanian
            "asia_mongolic", "asia_tungusic", "asia_sinotibetan_tibetic", "asia_hmongmien",
            "asia_vietic", "africa_afroasiatic_chadic", "americas_athabaskan",
            "americas_otomanguean", "americas_tupian", "americas_siouan",
            "eurasia_ie_hellenic", "eurasia_ie_baltic", "eurasia_ie_armenian", "eurasia_ie_albanian",
            // Third batch: previously listed gaps
            "caucasus_nakh_dagestanian", "africa_nigercongo_atlantic", "africa_nigercongo_gur",
            "africa_nigercongo_kwa", "africa_nilosaharan_saharan", "africa_songhai",
            "africa_afroasiatic_omotic", "eurasia_isolate_ainu", "eurasia_isolate_yeniseian",
            "oceania_australian_gunwinyguan", "eurasia_isolate_nivkh", "asia_isolate_burushaski",
            // Fourth batch: further families, creoles and constructed systems
            "americas_iroquoian", "americas_chibchan", "americas_panotacanan", "americas_je",
            "americas_aymaran", "americas_tucanoan", "eurasia_uralic_samoyedic",
            "eurasia_chukotko_kamchatkan", "asia_austronesian_malayo", "asia_austronesian_philippine",
            "oceania_austronesian_malagasy", "oceania_papuan_sepik", "africa_khoisan_tuu",
            "africa_nigercongo_ubangian", "creole_tok_pisin", "creole_haitian",
            "constructed_toki_pona", "constructed_esperanto",
            // Fifth batch: gaps found against Glottolog's family and isolate list
            "africa_khoisan_hadza", "africa_khoisan_sandawe", "asia_isolate_kusunda",
            "americas_wakashan", "americas_muskogean", "americas_caddoan", "americas_yuman",
            "americas_mixezoque", "americas_totonacan", "americas_mura", "americas_mapuche",
            "oceania_papuan_angan", "oceania_australian_nyulnyulan", "asia_austroasiatic_munda",
            "eurasia_ie_italic",
        ];
        let missing: Vec<&str> = required_phonology.iter().copied().filter(|k| !phono.contains_key(*k)).collect();
        assert!(missing.is_empty(), "missing phonology families: {:?}", missing);

        let morph = get_morphology_registry();
        for key in ["agglutinative", "fusional", "analytic", "polysynthetic", "root_pattern", "reduplicative", "noun_class_bantu", "infixing", "circumfixal", "polypersonal_agreement", "classifier"] {
            assert!(morph.contains_key(key), "missing morphology type '{}'", key);
        }

        let syntax = get_syntax_registry();
        for key in ["sov_ergative", "sov_split_ergative"] {
            assert!(syntax.contains_key(key), "missing alignment profile '{}'", key);
        }
        for key in WORD_ORDERS {
            let k = key.to_lowercase();
            assert!(syntax.contains_key(&k), "missing word order '{}'", k);
        }

        let sc = get_sound_change_registry();
        for key in ["lenition", "grimms_law", "final_devoicing", "nasal_assimilation", "vowel_reduction", "palatalization", "rhotacism", "cluster_simplification"] {
            assert!(sc.contains_key(key), "missing sound change set '{}'", key);
        }
    }
}
