extern crate if_chain;
use if_chain::if_chain;

extern crate matrix_sdk;
use matrix_sdk::Client;
use matrix_sdk::deserialized_responses::RoomEvent;
use matrix_sdk::room::Joined;
use matrix_sdk::ruma::api::client::context::get_context::v3 as get_context;
use matrix_sdk::ruma::events::{ AnySyncRoomEvent, AnySyncMessageLikeEvent, SyncMessageLikeEvent };
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;

// The context API is missing from the Matrix SDK
pub async fn get_events_before(
    event: &OriginalSyncRoomMessageEvent,
    client: &Client,
    room: &Joined
) -> Result<Vec<RoomEvent>, matrix_sdk::Error> {
    let request = get_context::Request::new(room.room_id(), &event.event_id);
    let http_response = client.send(request, None).await?;

    let mut response = Vec::with_capacity(http_response.events_before.len());

    for event in http_response.events_before {
        if_chain! {
            if let AnySyncRoomEvent::MessageLike(encrypted_event) = event.deserialize_as::<AnySyncRoomEvent>()?;
            if let AnySyncMessageLikeEvent::RoomEncrypted(encrypted_event) = encrypted_event;
            if let SyncMessageLikeEvent::Original(encrypted_event) = encrypted_event;
            then {
                let decrypted_event = room.decrypt_event(&encrypted_event).await?;
                response.push(decrypted_event);
            }
            else {
                response.push(RoomEvent { event, encryption_info: None });
            }
        }
    }

    Ok(response)
}

