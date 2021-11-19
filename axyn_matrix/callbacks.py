import asyncio
from nio import InviteMemberEvent, RoomMessageFormatted

def attach_callbacks(client, responder):
    """Attach all of Axyn's event callbacks to the client."""

    async def message_callback(room, event):
        """Function called when a message event is received."""

        # Ignore Axyn's own messages
        if event.sender == client.user_id:
            return

        asyncio.create_task(
            # Send a read receipt for this message
            client.room_read_markers(room.room_id, event.event_id, event.event_id)
        )

        response, distance = responder.get_response(event.body)

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
