"""
Definitions of linguistic archetypes and influences for the Esoterica generator.
"""

ARCHETYPES = {
    # Natural Language Families
    "indo_european": {
        "name": "Indo-European",
        "phonology": {
            "vowels": ["a", "e", "i", "o", "u"],
            "consonants": ["p", "b", "t", "d", "k", "g", "s", "z", "m", "n", "r", "l", "w", "j"],
            "syllable_structure": "CCCVCCCC", # High complexity
            "clusters": True
        },
        "morphology": "fusional",
        "syntax": "SVO",
        "features": ["cases", "gender", "ablaut"]
    },
    "sino_tibetan": {
        "name": "Sino-Tibetan",
        "phonology": {
            "vowels": ["a", "e", "i", "o", "u", "y"],
            "consonants": ["p", "t", "k", "s", "h", "m", "n", "ng", "l", "j", "w"],
            "syllable_structure": "CVC",
            "tones": 4,
            "clusters": False
        },
        "morphology": "analytic",
        "syntax": "SVO",
        "features": ["classifiers", "monosyllabic"]
    },
    "afroasiatic": {
        "name": "Afroasiatic",
        "phonology": {
            "vowels": ["a", "i", "u"], # Minimal vowel set (Semitic-like)
            "consonants": ["q", "k", "g", "t", "d", "s", "z", "sh", "h", "m", "n", "r", "l"],
            "pharyngeals": True,
            "syllable_structure": "CVC"
        },
        "morphology": "root_and_pattern",
        "syntax": "VSO",
        "features": ["triliteral_roots", "emphatic_consonants"]
    },
    "turkic": {
        "name": "Turkic",
        "phonology": {
            "vowels": ["a", "e", "i", "o", "u", "y", "oe", "ue"],
            "consonants": ["p", "b", "t", "d", "k", "g", "s", "z", "m", "n", "r", "l", "j"],
            "vowel_harmony": True,
            "syllable_structure": "CVC"
        },
        "morphology": "agglutinative",
        "syntax": "SOV",
        "features": ["no_gender", "postpositions"]
    },

    # Esoteric Archetypes
    "ithkuil_like": {
        "name": "Ithkuil-esque",
        "phonology": {
            "vowels": ["a", "e", "i", "o", "u", "y", "ae", "oe", "ue"],
            "consonants": ["p", "ph", "b", "bh", "t", "th", "d", "dh", "k", "kh", "g", "gh", "q", "qh", "s", "z", "sh", "zh", "m", "n", "r", "l", "x"],
            "syllable_structure": "CCCVCCCC",
            "extreme_complexity": True
        },
        "morphology": "polysynthetic",
        "syntax": "free",
        "features": ["semantic_density", "high_precision"]
    },
    "toki_pona_like": {
        "name": "Toki Pona-esque",
        "phonology": {
            "vowels": ["a", "e", "i", "o", "u"],
            "consonants": ["p", "t", "k", "s", "m", "n", "l", "j", "w"],
            "syllable_structure": "CV(N)",
            "clusters": False
        },
        "morphology": "minimalist",
        "syntax": "SVO",
        "features": ["small_lexicon", "semantic_simplicity"]
    },
    "lojban_like": {
        "name": "Lojban-esque",
        "phonology": {
            "vowels": ["a", "e", "i", "o", "u", "y"],
            "consonants": ["p", "b", "t", "d", "k", "g", "s", "z", "f", "v", "m", "n", "r", "l", "c", "j"],
            "syllable_structure": "CCVVC"
        },
        "morphology": "logical",
        "syntax": "predicate",
        "features": ["unambiguous", "formal_grammar"]
    },
    "musical": {
        "name": "Musical (Solresol-esque)",
        "phonology": {
            "vowels": ["do", "re", "mi", "fa", "sol", "la", "si"],
            "consonants": [],
            "syllable_structure": "V",
            "medium": "notes"
        },
        "morphology": "compositional",
        "syntax": "SVO",
        "features": ["musical_scale", "no_consonants"]
    }
}
