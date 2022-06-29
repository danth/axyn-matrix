import json
import re
import sys
from flipgenic import Responder, Message

print("Creating responder")
responder = Responder(sys.argv[2], "en_core_web_md")

# Matches scene numbers like "ACT I. SCENE III.", "ACT III", and "SCENE XV"
scene_regex = re.compile(r"(?:(?:ACT|SCENE) [IVX]+\.? ?)+$", re.MULTILINE)

# Matches lines like "CHARACTER. Lorem ipsum\n dolor"
line_regex = re.compile(r"^ ([A-Z ]+)\. ((?:[\S ]+\n?(?!^ [A-Z ]+\. ))+)", re.MULTILINE)

# Matches stage directions like "[lorem ipsum dolor]", "Exit CHARACTER", and "Exeunt"
stage_direction_regex = re.compile(r" ?(\[[^\)\]]+[\)\]]|(?:Exeunt|Exit[A-Z ]*)$) ?", re.MULTILINE)

whitespace_regex = re.compile(r"\s+")

def format_line(line):
    line_without_stage_directions = re.sub(stage_direction_regex, "", line)

    # Replace any newlines or double spaces with a single space
    return re.sub(whitespace_regex, " ", line_without_stage_directions)

def format_character(character, title):
    return json.dumps([
        {
            "type": "character",
            "name": character.title()
        },
        {
            "type": "title",
            "title": title
        },
        {
            "type": "author",
            "name": "William Shakespeare"
        },
        {
            "type": "dataset",
            "name": "BRIDGES Data",
            "link": "https://bridgesdata.herokuapp.com/api/datasets/shakespeare"
        }
    ])

with open(sys.argv[1]) as file:
    for play in json.load(file)["data"]:
        title = play["title"]
        print("Processing:", title)

        # The play must be split into scenes so that the conversation does not
        # run across scene boundaries
        scenes = re.split(scene_regex, play["text"])
        for scene in scenes:
            lines = [
                Message(
                    format_line(match.group(2)),
                    format_character(match.group(1), title)
                )
                for match
                in re.finditer(line_regex, scene)
            ]

            for line_a, line_b in zip(lines[:-1], lines[1:]):
                responder.add_response(line_a.text, line_b)

print("Saving responder")
responder.commit_responses()
