extern crate dirs;

extern crate matrix_sdk;
use matrix_sdk::Client;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::member::StrippedRoomMemberEvent;
use matrix_sdk::ruma::events::room::message::{
    FormattedBody, MessageFormat, MessageType,
    OriginalSyncRoomMessageEvent, RoomMessageEventContent, TextMessageEventContent
};

extern crate matrix_sdk_sled;
use matrix_sdk_sled::make_store_config;

extern crate serde;
use serde::{Serialize, Deserialize};

extern crate tokio;
use tokio::time::{sleep, Duration};

use crate::store::ResponseStore;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Body {
    plain: String,
    html: Option<String>
}

fn get_body(event: OriginalSyncRoomMessageEvent) -> Option<Body> {
    match event.content.msgtype {
        MessageType::Text(TextMessageEventContent {
            formatted: Some(FormattedBody {
                format: MessageFormat::Html,
                body: html
            }),
            body,
            ..
        })
            => Some(Body { plain: body, html: Some(html) }),

        MessageType::Text(TextMessageEventContent { body, .. })
            => Some(Body { plain: body, html: None }),

        _ => None
    }
}

async fn process_message(
    event: OriginalSyncRoomMessageEvent,
    client: Client,
    room: Room,
    Ctx(database): Ctx<ResponseStore>
) {
    // Don't respond to our own messages
    if event.sender == client.user_id().await.expect("Retrieving own user ID") { return; }

    if let Room::Joined(room) = room {
        if let Some(body) = get_body(event) {
            // TODO: Handle response errors
            if let Ok(response) = database.respond(&body.plain) {
                let response_content = match response.html {
                    Some(html) => RoomMessageEventContent::text_html(response.plain, html),
                    None => RoomMessageEventContent::text_plain(response.plain)
                };

                room.send(response_content, None).await.expect("Sending response");
            }
        }
    }
}

async fn join_on_invite(
    room_member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
) {
    // Only respond to invites for ourself
    if room_member.state_key != client.user_id().await.expect("Retrieving own user ID") { return; }

    if let Room::Invited(room) = room {
        println!("Joining room {}", room.room_id());

        let mut delay = 2;
        while let Err(err) = room.accept_invitation().await {
            // Retry joining due to https://github.com/matrix-org/synapse/issues/4345

            eprintln!("Failed to join room {}, retrying in {}s", room.room_id(), delay);

            sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                eprintln!("Couldn't join room {}: {}", room.room_id(), err);
                return;
            }
        }

        println!("Successfully joined room {}", room.room_id());
    }
}

pub async fn login_and_sync(
    homeserver_url: String,
    username: &str,
    password: &str,
    device_id: &str
) -> anyhow::Result<()> {
    let path = dirs::home_dir().expect("Finding home directory");
    let store_config = make_store_config(path, None)?;

    let client = Client::builder()
        .homeserver_url(homeserver_url)
        .store_config(store_config)
        .build().await?;

    let database = ResponseStore::load().expect("Loading store");
    database.insert("fish", Body { plain: "Today's fish is trout á la créme".to_string(), html: None });
    client.register_event_handler_context(database);

    client.login(username, password, Some(device_id), Some("Axyn")).await?;
    println!("Connected to Matrix as {}", username);

    client
        .register_event_handler(process_message).await
        .register_event_handler(join_on_invite).await;

    client.sync(SyncSettings::default()).await;

    Ok(())
}

