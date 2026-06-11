from esoterica.phonology import PhonologyEngine

def test_phonology():
    print("Testing Phonology Engine:")
    for archetype in ["indo_european", "toki_pona_like", "musical"]:
        engine = PhonologyEngine(archetype)
        print(f"\nArchetype: {archetype}")
        print(f"Sample words: {[engine.generate_word() for _ in range(5)]}")

if __name__ == "__main__":
    test_phonology()
