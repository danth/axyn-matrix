extern crate matrix_sdk;
use matrix_sdk::Client;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::member::StrippedRoomMemberEvent;

extern crate matrix_sdk_sled;
use matrix_sdk_sled::StateStore;

extern crate tokio;
use tokio::time::{sleep, Duration};

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
    let state_store = StateStore::open_with_path("matrix_state")?;

    let client = Client::builder()
        .homeserver_url(homeserver_url)
        .state_store(Box::new(state_store))
        .build().await?;

    client.login(username, password, Some(device_id), Some("Axyn")).await?;
    println!("Connected to Matrix as {}", username);

    client.register_event_handler(join_on_invite).await;

    client.sync(SyncSettings::default()).await;

    Ok(())
}

