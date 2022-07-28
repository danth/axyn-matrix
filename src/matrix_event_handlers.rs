extern crate dirs;

extern crate futures;
use futures::try_join;

extern crate matrix_sdk;
use matrix_sdk::{
    config::SyncSettings,
    event_handler::Ctx,
    room::{Joined, Room},
    ruma::events::room::{
        member::StrippedRoomMemberEvent,
        message::{OriginalSyncRoomMessageEvent, RoomMessageEventContent},
    },
    Account,
    Client,
};

extern crate matrix_sdk_sled;
use matrix_sdk_sled::make_store_config;

extern crate mime;

extern crate tokio;
use tokio::time::{sleep, Duration};

use std::fs::File;

use crate::{
    matrix_body::{get_previous_body, Body, HasBody},
    store::ResponseStore,
};

async fn send_response(body: &Body, room: &Joined, database: &ResponseStore) -> anyhow::Result<()> {
    if let Ok(response) = database.respond(&body.plain) {
        let response_content = match response.html {
            Some(html) => RoomMessageEventContent::text_html(response.plain, html),
            None => RoomMessageEventContent::text_plain(response.plain),
        };

        room.send(response_content, None).await?;
    }

    Ok(())
}

async fn learn_from_message(
    body: Body,
    event: &OriginalSyncRoomMessageEvent,
    client: &Client,
    room: &Joined,
    database: &ResponseStore,
) -> anyhow::Result<()> {
    let previous_body = get_previous_body(event, client, room).await?;

    if let Some(previous_body) = previous_body {
        database.insert(&previous_body.plain, body)?;
    }

    Ok(())
}

async fn process_message(
    event: OriginalSyncRoomMessageEvent,
    client: Client,
    room: Room,
    Ctx(database): Ctx<ResponseStore>,
) {
    // Don't respond to our own messages
    if event.sender == client.user_id().await.expect("Getting own user ID") {
        return;
    }

    if let Room::Joined(room) = room {
        if let Some(body) = event.get_body() {
            if let Err(error) = send_response(&body, &room, &database).await {
                eprintln!("Error sending response: {}", error);
            }

            if let Err(error) = learn_from_message(body.clone(), &event, &client, &room, &database).await {
                eprintln!("Error learning from message: {}", error);
            }
        }

        if let Err(error) = room.read_receipt(&event.event_id).await {
            eprintln!("Error setting read receipt: {}", error);
        }
    }
}

async fn join_on_invite(room_member: StrippedRoomMemberEvent, client: Client, room: Room) {
    // Only respond to invites for ourself
    if room_member.state_key != client.user_id().await.expect("Getting own user ID") {
        return;
    }

    if let Room::Invited(room) = room {
        println!("Joining room {}", room.room_id());

        let mut delay = 2;
        while let Err(error) = room.accept_invitation().await {
            // Retry joining due to https://github.com/matrix-org/synapse/issues/4345

            eprintln!(
                "Failed to join room {}, retrying in {}s",
                room.room_id(),
                delay
            );

            sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                eprintln!("Failed to join room {}: {}", room.room_id(), error);
            }
        }

        println!("Successfully joined room {}", room.room_id());
    }
}

async fn set_display_name(account: &Account) -> anyhow::Result<()> {
    if account.get_display_name().await? != Some("Axyn".to_string()) {
        println!("Setting display name");
        account.set_display_name(Some("Axyn")).await?;
    }

    Ok(())
}

async fn set_avatar(account: &Account) -> anyhow::Result<()> {
    if account.get_avatar_url().await? == None {
        println!("Setting avatar");
        let mut image = File::open(env!("AVATAR_PNG"))?;
        account.upload_avatar(&mime::IMAGE_PNG, &mut image).await?;
    }

    Ok(())
}

async fn sync(client: &Client) -> anyhow::Result<()> {
    client
        .register_event_handler(process_message)
        .await
        .register_event_handler(join_on_invite)
        .await;

    println!("Listening for events");
    client.sync(SyncSettings::default()).await;

    Ok(())
}

pub async fn login_and_sync(
    homeserver_url: String,
    username: &str,
    password: &str,
    device_id: &str,
) -> anyhow::Result<()> {
    let path = dirs::home_dir().expect("Finding home directory");
    let store_config = make_store_config(path, None)?;

    let client = Client::builder()
        .homeserver_url(homeserver_url)
        .store_config(store_config)
        .build()
        .await?;

    let database = ResponseStore::load()?;
    client.register_event_handler_context(database);

    client
        .login(username, password, Some(device_id), Some("Axyn"))
        .await?;
    println!("Connected to Matrix as {}", username);

    let account = &client.account();
    try_join!(set_display_name(account), set_avatar(account), sync(&client))?;

    Ok(())
}
