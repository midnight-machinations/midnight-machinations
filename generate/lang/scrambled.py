
import json
import random
import re
import sys
import os
sys.path.append(os.path.abspath('..'))

from lang import CLIENT_LANG_ROOT, ENGLISH_LANG_FILE_PATH

SCRAMBLED_LANG_FILE_PATH = f"{CLIENT_LANG_ROOT}/dyslexic.json"

def scramble_word(word: str, cache: dict[str, str]) -> str:
    if word.lower() not in cache.keys():
        letters = list(word.lower())
        random.shuffle(letters)
        shuffle = "".join(letters)
        cache[word.lower()] = shuffle
    
    capitalized_letters = [letter.isupper() for letter in word]
    map_case = lambda idx, letter: letter.upper() if capitalized_letters[idx] else letter.lower()
    return "".join([map_case(idx, letter) for (idx, letter) in enumerate(cache[word.lower()])])


def scramble(key: str, value: str, cache: dict[str, str]) -> str:
    if key == "language":
        return "Scrambled"
    
    random.seed(key)
    
    return "".join([
        scramble_word(word, cache) 
        for word in re.findall(r"[\w'-]+|\W", value)
        ])


def main():
    cache={}

    with open(ENGLISH_LANG_FILE_PATH, "r") as english_lang_file:
        lang = json.loads(english_lang_file.read())

        lang_dict: dict[str, str] = dict(lang)

        scrambled_dict: dict[str, str] = {}

        # Load the existing scrambled language data
        with open(SCRAMBLED_LANG_FILE_PATH, "r") as scrambled_lang_file:
            scrambled_dict = json.loads(scrambled_lang_file.read())
            cache = scrambled_dict.copy()

        for (key, value) in lang_dict.items():
            scrambled_dict[key] = scramble(key, value, cache=cache)

        with open(SCRAMBLED_LANG_FILE_PATH, "w+") as scrambled_lang_file:
            scrambled_lang_file.write(json.dumps(scrambled_dict, indent=4))



if __name__ == "__main__":
    main()