import asyncio
import os
import subprocess

import aiofiles
from aiohttp import ClientConnectionError, ServerDisconnectedError
from flipgenic import Responder
from nio import AsyncClient, AsyncClientConfig, LoginError,  ProfileGetAvatarError, UploadResponse
import spacy

from axyn_matrix.callbacks import attach_callbacks


def create_client():
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

    return client


def load_responders():
    """Load the Flipgenic responders responsible for selection Axyn's messages."""

    print("Loading SpaCy model")
    model = spacy.load("en_core_web_md", exclude=["ner"])

    print("Loading Flipgenic responders")

    read_only = Responder("@INITIAL_RESPONDER@", model)

    writeable_path = os.path.join(os.environ["AXYN_MATRIX_STORE_PATH"], "responder")
    writeable = Responder(writeable_path, model)

    return (read_only, writeable)


def setup_client_responder():
    """Create and link the Matrix client and Flipgenic responder."""

    client = create_client()

    responders = load_responders()

    attach_callbacks(client, responders)

    return client


async def set_avatar(client):
    """Set Axyn's profile picture to axyn-icon.png, if one is not already set."""

    current_avatar = await client.get_avatar()
    if type(current_avatar) != ProfileGetAvatarError:
        # A profile picture is already set
        return

    AVATAR = os.path.join(os.path.dirname(__file__), "images/axyn-icon.png")

    # Upload the image to the homeserver
    avatar_stat = os.stat(AVATAR)
    async with aiofiles.open(AVATAR, "rb") as avatar:
        upload_response, _ = await client.upload(
            avatar,
            content_type="image/png",
            filename="axyn-icon.png",
            filesize=avatar_stat.st_size
        )

    # Set the uploaded image as our avatar
    if type(upload_response) == UploadResponse:
        await client.set_avatar(upload_response.content_uri)

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

    asyncio.create_task(client.set_displayname("Axyn"))
    asyncio.create_task(set_avatar(client))

    await client.sync_forever(timeout=30000, full_state=True, set_presence="online")


async def loop():
    client = setup_client_responder()

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
