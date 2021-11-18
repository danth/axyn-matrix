import ast
import sys
from os import path
from flipgenic import Responder, Message

print("Creating responder")
responder = Responder(sys.argv[2], "en_core_web_md")

print("Loading characters")
with open(path.join(sys.argv[1], "movie_characters_metadata.txt"), "rb") as file:
    characters = {}
    for row in file.readlines():
        row_decoded = row.decode("latin").split(" +++$+++ ")

        character_id = row_decoded[0]
        character_name = row_decoded[1].title()
        movie_title = row_decoded[3].title()

        characters[character_id] = f"{character_name}, {movie_title}"

print("Loading lines")
with open(path.join(sys.argv[1], "movie_lines.txt"), "rb") as file:
    lines = {}
    for row in file.readlines():
        row_decoded = row.decode("latin").split(" +++$+++ ")

        line_id = row_decoded[0]
        line_text = row_decoded[4].strip()
        line_character = characters[row_decoded[1]]

        lines[line_id] = Message(line_text, line_character)

with open(path.join(sys.argv[1], "movie_conversations.txt"), "rb") as file:
    for row_number, row in enumerate(file.readlines()):
        if row_number % 1000 == 0:
            print("Done", row_number, "conversations")

        row_decoded = row.decode("latin").split(" +++$+++ ")

        line_ids = ast.literal_eval(row_decoded[3].strip())
        conversation_lines = [lines.pop(line_id) for line_id in line_ids]

        for line_a, line_b in zip(conversation_lines[:-1], conversation_lines[1:]):
            responder.add_response(line_a.text, line_b)

print("Saving responder")
responder.commit_responses()
