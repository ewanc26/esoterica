import json
from esoterica.phonology import PhonologyEngine
from esoterica.morphology import MorphologyEngine

class LexiconGenerator:
    def __init__(self, archetype_key):
        self.phonology = PhonologyEngine(archetype_key)
        self.morphology = MorphologyEngine(archetype_key, self.phonology)
        self.lexicon = {}

    def generate_core_lexicon(self, size=100):
        """Generates a core vocabulary."""
        for i in range(size):
            word = self.phonology.generate_word()
            self.lexicon[word] = f"definition_{i}"
        return self.lexicon

    def save_to_file(self, filename):
        with open(filename, 'w') as f:
            json.dump(self.lexicon, f, indent=4)
