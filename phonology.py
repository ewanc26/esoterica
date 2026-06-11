import random
from esoterica.archetypes import ARCHETYPES

class PhonologyEngine:
    def __init__(self, archetype_key):
        self.archetype = ARCHETYPES.get(archetype_key)
        if not self.archetype:
            raise ValueError(f"Unknown archetype: {archetype_key}")
        self.phonology = self.archetype["phonology"]

    def generate_syllable(self):
        """Generates a syllable based on the archetype's structure."""
        structure = self.phonology.get("syllable_structure", "CVC")
        
        syllable = ""
        for char in structure:
            if char == 'C':
                syllable += random.choice(self.phonology.get("consonants", []))
            elif char == 'V':
                syllable += random.choice(self.phonology.get("vowels", []))
            elif char == 'N':
                syllable += random.choice(["n", "m", "ng"]) # Nasal coda
        
        return syllable

    def generate_word(self, num_syllables=2):
        """Generates a word with the specified number of syllables."""
        return "".join([self.generate_syllable() for _ in range(num_syllables)])
