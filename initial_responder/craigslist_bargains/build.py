import json
import sys
from flipgenic import Responder, Message

print("Creating responder")
responder = Responder(sys.argv[2], "en_core_web_md")

with open(sys.argv[1]) as file:
    for conversation_number, conversation in enumerate(json.load(file)):
        if conversation_number % 1000 == 0:
            print("Done", conversation_number, "conversations")

        messages = [
            Message(event["data"], "Craigslist")
            for event
            in conversation["events"]
            if event["action"] == "message"
        ]

        for message_a, message_b in zip(messages[:-1], messages[1:]):
            responder.add_response(message_a.text, message_b)

print("Saving responder")
responder.commit_responses()
