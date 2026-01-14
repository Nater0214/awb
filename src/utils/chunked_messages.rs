use std::pin::Pin;

use anyhow::Context as _;
use async_stream::try_stream;
use serenity::all::{CacheHttp, ChannelId, GetMessages, Message};
use tokio_stream::Stream;

pub(crate) struct ChunkedMessageGenerator<'a, H> {
    pub chunk_size: u8,
    pub channel: ChannelId,
    pub http: &'a H,
}

impl<'a, H> ChunkedMessageGenerator<'a, H>
where
    H: CacheHttp,
{
    pub fn new(chunk_size: u8, channel: impl AsRef<ChannelId>, http: &'a H) -> Self {
        Self {
            chunk_size,
            channel: channel.as_ref().to_owned(),
            http,
        }
    }

    pub fn stream(&self) -> Pin<Box<impl Stream<Item = anyhow::Result<Message>>>> {
        let chunk_size = self.chunk_size;
        let channel = self.channel;
        let http = self.http;

        Box::pin(try_stream! {
            let mut last_id = None;
            loop {
                let mut get_messages = GetMessages::new().limit(chunk_size);
                if let Some(last_id) = last_id {
                    get_messages = get_messages.before(last_id);
                }

                let messages = channel
                    .messages(http, get_messages)
                    .await
                    .context("Failed to get messages")?;

                if messages.is_empty() {
                    break;
                }

                last_id = messages.last().map(|message| message.id);

                for message in messages {
                    yield message;
                }
            }
        })
    }
}
