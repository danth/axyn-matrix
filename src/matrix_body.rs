extern crate matrix_sdk;
use matrix_sdk::Client;
use matrix_sdk::room::Joined;
use matrix_sdk::ruma::events::{ AnyRoomEvent, AnyMessageLikeEvent, MessageLikeEvent };
use matrix_sdk::ruma::events::room::message::{
    FormattedBody, MessageFormat, MessageType, OriginalSyncRoomMessageEvent, Relation,
    TextMessageEventContent
};
use matrix_sdk::ruma::serde::Raw;

extern crate serde;
use serde::{Serialize, Deserialize};

use crate::matrix_api::get_events_before;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Body {
    pub plain: String,
    pub html: Option<String>
}

pub fn get_body_from_event(event: &OriginalSyncRoomMessageEvent) -> Option<Body> {
    match &event.content.msgtype {
        MessageType::Text(TextMessageEventContent {
            formatted: Some(FormattedBody {
                format: MessageFormat::Html,
                body: html
            }),
            body,
            ..
        })
            => Some(Body {
                plain: body.to_string(),
                html: Some(html.to_string())
            }),

        MessageType::Text(TextMessageEventContent { body, .. })
            => Some(Body {
                plain: body.to_string(),
                html: None
            }),

        _ => None
    }
}

pub fn get_body_from_raw(raw: &Raw<AnyRoomEvent>) -> Option<Body> {
    if let Ok(
        AnyRoomEvent::MessageLike(
            AnyMessageLikeEvent::RoomMessage(
                MessageLikeEvent::Original(event)))) = raw.deserialize() {
        get_body_from_event(&event.into())
    } else {
        None
    }
}

pub async fn get_previous_body(
    event: &OriginalSyncRoomMessageEvent,
    client: &Client,
    room: &Joined
) -> Result<Option<Body>, matrix_sdk::Error> {
    // Look for explicit replies first
    if let Some(Relation::Reply{ in_reply_to }) = &event.content.relates_to {
        let previous_event = room.event(&in_reply_to.event_id).await?;
        if let Some(previous_body) = get_body_from_raw(&previous_event.event) {
            return Ok(Some(previous_body));
        }
    }

    // Fall back to chronological order
    let events_before = get_events_before(event, client, room).await?;
    // We must check each event until we find one which is a text message
    for previous_event in events_before.iter() {
        if let Some(previous_body) = get_body_from_raw(&previous_event.event) {
            return Ok(Some(previous_body));
        }
    }

    Ok(None)
}
