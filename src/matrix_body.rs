extern crate matrix_sdk;
use matrix_sdk::{
    room::Joined,
    ruma::{
        events::{
            room::message::{
                FormattedBody,
                MessageFormat,
                MessageType,
                OriginalRoomMessageEvent,
                OriginalSyncRoomMessageEvent,
                Relation,
                RoomMessageEventContent,
                TextMessageEventContent,
            },
            AnyMessageLikeEvent,
            AnyRoomEvent,
            MessageLikeEvent,
        },
        serde::Raw,
    },
    Client,
};

extern crate serde;
use serde::{Deserialize, Serialize};

use crate::matrix_api::get_events_before;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Body {
    pub plain: String,
    pub html: Option<String>,
}

pub trait HasBody {
    fn get_body(&self) -> Option<Body>;
}

impl HasBody for TextMessageEventContent {
    fn get_body(&self) -> Option<Body> {
        match self {
            TextMessageEventContent {
                formatted:
                    Some(FormattedBody {
                        format: MessageFormat::Html,
                        body: html,
                    }),
                body,
                ..
            } => Some(Body {
                plain: body.to_string(),
                html: Some(html.to_string()),
            }),

            TextMessageEventContent { body, .. } => Some(Body {
                plain: body.to_string(),
                html: None,
            })
        }
    }
}

impl HasBody for OriginalRoomMessageEvent {
    fn get_body(&self) -> Option<Body> {
        match &self.content.msgtype {
            MessageType::Text(content) => content.get_body(),
            _ => None
        }
    }
}

impl HasBody for OriginalSyncRoomMessageEvent {
    fn get_body(&self) -> Option<Body> {
        match &self.content.msgtype {
            MessageType::Text(content) => content.get_body(),
            _ => None
        }
    }
}

impl HasBody for MessageLikeEvent<RoomMessageEventContent> {
    fn get_body(&self) -> Option<Body> {
        match self {
            MessageLikeEvent::Original(event) => event.get_body(),
            _ => None
        }
    }
}

impl HasBody for AnyMessageLikeEvent {
    fn get_body(&self) -> Option<Body> {
        match self {
            AnyMessageLikeEvent::RoomMessage(event) => event.get_body(),
            _ => None
        }
    }
}

impl HasBody for AnyRoomEvent {
    fn get_body(&self) -> Option<Body> {
        match self {
            AnyRoomEvent::MessageLike(event) => event.get_body(),
            _ => None
        }
    }
}

impl HasBody for Raw<AnyRoomEvent> {
    fn get_body(&self) -> Option<Body> {
        match self.deserialize() {
            Ok(event) => event.get_body(),
            Err(_) => None
        }
    }
}

pub async fn get_previous_body(
    event: &OriginalSyncRoomMessageEvent,
    client: &Client,
    room: &Joined,
) -> Result<Option<Body>, matrix_sdk::Error> {
    // Look for explicit replies first
    if let Some(Relation::Reply { in_reply_to }) = &event.content.relates_to {
        let previous_event = room.event(&in_reply_to.event_id).await?;
        if let Some(previous_body) = previous_event.event.get_body() {
            return Ok(Some(previous_body));
        }
    }

    // Fall back to chronological order
    let events_before = get_events_before(event, client, room).await?;
    // We must check each event until we find one which is a text message
    for previous_event in events_before.iter() {
        if let Some(previous_body) = previous_event.event.get_body() {
            return Ok(Some(previous_body));
        }
    }

    Ok(None)
}
