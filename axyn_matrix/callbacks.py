import random

from flipgenic import Message
from nio import InviteMemberEvent, RoomMessageFormatted, JoinError, RoomContextResponse


def generate_response(responders, event):
    """Use Flipgenic to select a response to the given event."""

    # First try a response from the learned responses
    response, uncertainty = responders[1].get_response(event.body)

    # Now try a response from the initial dataset
    initial_response, initial_uncertainty = responders[0].get_response(event.body)

    # Use the more certain response
    if uncertainty < initial_uncertainty:
        return (response, uncertainty, True)
    else:
        return (initial_response, initial_uncertainty, False)


def response_probability(uncertainty, member_count):
    """
    Return the probability of sending a reply.

    This is calculated based on the number of people in the room, and the
    uncertainty value returned by Flipgenic.
    """

    # This number controls how the probability decreases
    # as the number of people in the room increases.
    TALKATIVITY = 4

    # This number controls how the probability decreases
    # as the uncertainty value increases.
    CONFIDENCE = 8

    # https://www.math3d.org/Im8hCdq80
    certain_probability = min(1, (TALKATIVITY + 2) / (TALKATIVITY + member_count))
    uncertainty_deduction = uncertainty / CONFIDENCE
    probability = max(0, certain_probability - uncertainty_deduction)

    return probability


def format_reply(response, metadata_is_user_id=False):
    """Convert a Flipgenic response into a Matrix event."""

    formatted_body = f"{response.text}<br><sub>"

    if metadata_is_user_id:
        # Create a mention pill
        link = "https://matrix.to/#/" + response.metadata
        text = response.metadata.split(":")[0]
        formatted_body += f"<a href=\"{link}\">{text}</a>"
    else:
        formatted_body += response.metadata

    formatted_body += "</sub>"

    return {
        "msgtype": "m.text",
        "body": response.text,
        "format": "org.matrix.custom.html",
        "formatted_body": formatted_body
    }


async def learn_from_message(responders, client, room, event):
    """Add a recieved message to the writeable Flipgenic responder."""

    # Retrieve a few events which happened immediately before the given event
    context = await client.room_context(room.room_id, event.event_id)
    if isinstance(context, RoomContextResponse):

        # Find the closest message in the history
        # (There could be other types of event in between, which we ignore)
        for event_before in context.events_before:
            if isinstance(event_before, RoomMessageFormatted):

                # Don't learn from consecutive messages from the same person
                if event_before.sender != event.sender:

                    responders[1].learn_response(
                        event_before.body,
                        Message(event.body, event.sender)
                    )
                    break


def attach_callbacks(client, responders):
    """Attach all of Axyn's event callbacks to the client."""

    async def message_callback(room, event):
        """Function called when a message event is received."""

        # Ignore Axyn's own messages
        if event.sender == client.user_id:
            return

        (response, uncertainty, metadata_is_user_id) = generate_response(responders, event)

        if random.random() > response_probability(uncertainty, room.member_count):
            await client.room_read_markers(room.room_id, event.event_id, event.event_id)
        else:
            await client.room_send(
                room.room_id,
                "m.room.message",
                format_reply(response, metadata_is_user_id),
                ignore_unverified_devices=True
            )

        await learn_from_message(responders, client, room, event)

    client.add_event_callback(message_callback, RoomMessageFormatted)

    async def invite_callback(room, event):
        """Function called when an invite event is received."""

        if event.state_key == client.user_id:
            result = await client.join(room.room_id)
            if type(result) == JoinError:
                print("Failed to join room", room.room_id, "-", result.message)
            else:
                print("Joined room", room.room_id)

    client.add_event_callback(invite_callback, InviteMemberEvent)
