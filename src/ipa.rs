//! Orthography-to-IPA transcription.
//!
//! The generator writes words in a romanised practical orthography (`sh`,
//! `ng`, `kw`, …). This module turns that back into a narrow-ish IPA string
//! using a single left-to-right longest-match pass, so multigraphs win over
//! their component letters and no mapping can clobber an earlier one.
//!
//! It is a transcription convention, not a phonetic analyser: it assumes the
//! romanisation used by `data/phonologies.toml` and passes unknown characters
//! through unchanged.

/// Romanisation → IPA correspondences, consulted longest-first.
///
/// Order within the table does not matter; `transcribe` always prefers the
/// longest key that matches at the current position.
const TRANSCRIPTIONS: &[(&str, &str)] = &[
    // ── Trigraphs ────────────────────────────────────────────────────────
    ("tsh", "t\u{0283}"),
    ("dzh", "d\u{0292}"),
    ("ngw", "\u{014b}\u{02b7}"),
    // ── Digraphs ─────────────────────────────────────────────────────────
    ("ch", "t\u{0283}"),
    ("dh", "\u{00f0}"),
    ("dz", "dz"),
    ("gh", "\u{0263}"),
    ("gw", "\u{0261}\u{02b7}"),
    ("hw", "\u{028d}"),
    ("kh", "x"),
    ("kw", "k\u{02b7}"),
    ("ll", "\u{026c}"),
    ("ny", "\u{0272}"),
    ("ph", "f"),
    ("rr", "r"),
    ("sh", "\u{0283}"),
    ("th", "\u{03b8}"),
    ("ts", "ts"),
    ("zh", "\u{0292}"),
    ("ng", "\u{014b}"),
    // ── Single consonants ────────────────────────────────────────────────
    ("c", "k"),
    ("g", "\u{0261}"),
    ("j", "j"),
    ("q", "q"),
    ("r", "\u{027e}"),
    ("x", "ks"),
    ("'", "\u{0294}"),
    // ── Vowels needing a distinct IPA symbol ─────────────────────────────
    ("\u{00e4}", "\u{00e6}"),
    ("\u{00f6}", "\u{00f8}"),
    ("\u{00fc}", "y"),
    ("\u{00e5}", "\u{0254}"),
    ("\u{0151}", "\u{00f8}"),
    ("\u{0171}", "y"),
    // ── Tone superscripts → Chao tone letters ────────────────────────────
    ("\u{00b9}", "\u{02e9}"),
    ("\u{00b2}", "\u{02e8}"),
    ("\u{00b3}", "\u{02e7}"),
    ("\u{2074}", "\u{02e6}"),
    ("\u{2075}", "\u{02e5}"),
    // ── Orthographic boundaries carry no phonetic content ────────────────
    ("-", ""),
];

/// Longest multigraph in the table, used to bound the match window.
fn max_key_chars() -> usize {
    TRANSCRIPTIONS
        .iter()
        .map(|(from, _)| from.chars().count())
        .max()
        .unwrap_or(1)
}

/// Transcribe a romanised word into bare IPA, without enclosing slashes.
///
/// Characters with no correspondence — including IPA symbols already present
/// in a phoneme inventory — are copied through verbatim.
pub fn transcribe(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let window = max_key_chars();
    let mut out = String::with_capacity(word.len());
    let mut i = 0;

    while i < chars.len() {
        let mut matched = false;
        let upper = window.min(chars.len() - i);
        // Longest match wins, so "ng" beats "n" and "tsh" beats "ts".
        for len in (1..=upper).rev() {
            let candidate: String = chars[i..i + len].iter().collect();
            if let Some((_, to)) = TRANSCRIPTIONS.iter().find(|(from, _)| *from == candidate) {
                out.push_str(to);
                i += len;
                matched = true;
                break;
            }
        }
        if !matched {
            out.push(chars[i]);
            i += 1;
        }
    }

    out
}

/// Transcribe and wrap in phonemic slashes, e.g. `/kaˈta/`.
pub fn transcribe_phonemic(word: &str) -> String {
    format!("/{}/", transcribe(word))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_letters_pass_through() {
        assert_eq!(transcribe("pata"), "pata");
    }

    #[test]
    fn vowels_are_not_clobbered() {
        // The previous implementation rewrote every "a" as "æ" and "o" as "ø".
        assert_eq!(transcribe("mano"), "mano");
    }

    #[test]
    fn digraphs_beat_their_letters() {
        assert_eq!(transcribe("sha"), "\u{0283}a");
        assert_eq!(transcribe("nga"), "\u{014b}a");
        assert_eq!(transcribe("kwa"), "k\u{02b7}a");
    }

    #[test]
    fn trigraphs_beat_digraphs() {
        assert_eq!(transcribe("tsha"), "t\u{0283}a");
        assert_eq!(transcribe("tsa"), "tsa");
    }

    #[test]
    fn g_becomes_script_g() {
        assert_eq!(transcribe("gag"), "\u{0261}a\u{0261}");
    }

    #[test]
    fn front_rounded_vowels_are_mapped() {
        assert_eq!(transcribe("k\u{00e4}\u{00f6}"), "k\u{00e6}\u{00f8}");
    }

    #[test]
    fn tone_marks_become_chao_letters() {
        assert_eq!(transcribe("ma\u{00b3}"), "ma\u{02e7}");
    }

    #[test]
    fn morpheme_hyphens_are_dropped() {
        assert_eq!(transcribe("ta-ta"), "tata");
    }

    #[test]
    fn phonemic_form_is_wrapped() {
        let ipa = transcribe_phonemic("mana");
        assert!(ipa.starts_with('/') && ipa.ends_with('/'));
    }

    #[test]
    fn empty_input_is_empty() {
        assert_eq!(transcribe(""), "");
    }

    #[test]
    fn existing_ipa_symbols_survive() {
        assert_eq!(transcribe("\u{0294}a\u{0295}"), "\u{0294}a\u{0295}");
    }
}
