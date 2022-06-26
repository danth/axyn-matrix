import random

from nio import InviteMemberEvent, RoomMessageFormatted, JoinError


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


def attach_callbacks(client, responder):
    """Attach all of Axyn's event callbacks to the client."""

    async def message_callback(room, event):
        """Function called when a message event is received."""

        # Ignore Axyn's own messages
        if event.sender == client.user_id:
            return

        response, uncertainty = responder.get_response(event.body)

        if random.random() > response_probability(uncertainty, room.member_count):
            # Send a read receipt
            await client.room_read_markers(room.room_id, event.event_id, event.event_id)
        else:
            # Send a reply (sending a message also updates the read receipt)
            content = {
                "msgtype": "m.text",
                "body": response.text,
                "format": "org.matrix.custom.html",
                "formatted_body": f"{response.text}<br><sub>{response.metadata}</sub>",
            }

            await client.room_send(
                room.room_id,
                "m.room.message",
                content,
                ignore_unverified_devices=True
            )

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
