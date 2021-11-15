import asyncio
import os

from aiohttp import ClientConnectionError, ServerDisconnectedError
from nio import (
    AsyncClient,
    AsyncClientConfig,
    InviteMemberEvent,
    JoinError,
    LocalProtocolError,
    LoginError,
    RoomMessageText,
)


async def create_client():
    """Set up the the Matrix client."""

    client_config = AsyncClientConfig(
        max_limit_exceeded=0,
        max_timeouts=0,
        store_sync_tokens=True,
        encryption_enabled=True,
    )

    store_path = os.path.join(os.environ["AXYN_MATRIX_STORE_PATH"], "matrix")
    os.makedirs(store_path, exist_ok=True)

    client = AsyncClient(
        os.environ["AXYN_MATRIX_HOMESERVER"],
        os.environ["AXYN_MATRIX_USER_ID"],
        device_id=os.environ["AXYN_MATRIX_DEVICE_ID"],
        store_path=store_path,
        config=client_config,
    )

    # Set up event callbacks
    # client.add_event_callback(callbacks.message, (RoomMessageText,))

    async def invite_callback(room, event):
        """Function called when an invite event is received."""

        if event.state_key == client.user_id:
            result = await client.join(room.room_id)
            if type(result) == JoinError:
                print("Failed to join room", room.room_id, "-", result.message)
            else:
                print("Joined room", room.room_id)

    client.add_event_callback(invite_callback, InviteMemberEvent)

    return client


class FailedLogin(Exception):
    """Exception raised when the login credentials are incorrect."""


async def connect(client):
    """Log in and sync with the homeserver."""
    login_response = await client.login(
        password=os.environ["AXYN_MATRIX_USER_PASSWORD"],
        device_name=os.environ["AXYN_MATRIX_DEVICE_NAME"],
    )

    if type(login_response) == LoginError:
        raise FailedLogin(login_response.message)

    print(f"Logged in as {client.user_id}")

    await client.sync_forever(timeout=30000, full_state=True, set_presence="online")


async def loop():
    client = await create_client()

    # Keep trying to reconnect on failure
    reconnect = True
    while reconnect:
        try:
            await connect(client)

        except (ClientConnectionError, ServerDisconnectedError):
            print("Unable to connect to homeserver, retrying in 15s")

            # Sleep so we don't bombard the server with login requests
            await asyncio.sleep(15)

        except asyncio.TimeoutError:
            # Syncing with the homeserver may time out occasionally if:
            # - There are no new events to sync in the timeout period.
            # - The server is taking a long time to respond to the request.
            # In both cases it is fine to just try again.
            pass

        except FailedLogin as message:
            print("Failed to login -", message)
            reconnect = False

        finally:
            # Make sure to close the connection gracefully
            await client.close()


def main():
    """Run the loop function in an asyncio event loop."""
    asyncio.get_event_loop().run_until_complete(loop())
