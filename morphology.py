from esoterica.archetypes import ARCHETYPES

class MorphologyEngine:
    def __init__(self, archetype_key, phonology_engine):
        self.archetype = ARCHETYPES.get(archetype_key)
        self.phonology = phonology_engine
        self.type = self.archetype["morphology"]

    def create_word(self, base_root, grammatical_features):
        """Forms a word based on the archetype's morphology."""
        if self.type == "agglutinative":
            return self._agglutinate(base_root, grammatical_features)
        elif self.type == "root_and_pattern":
            return self._root_and_pattern(base_root, grammatical_features)
        # Add other types as needed
        return base_root

    def _agglutinate(self, root, features):
        """Adds suffixes to a root."""
        return root + "".join(features)

    def _root_and_pattern(self, root, pattern_type):
        """Applies a vowel pattern to a consonantal root."""
        # Simple implementation
        consonants = [c for c in root if c not in ['a', 'e', 'i', 'o', 'u']]
        if pattern_type == "past":
            return f"{consonants[0]}a{consonants[1]}a{consonants[2]}"
        return root
