use std::fmt::Display;
use serenity::client::Context;
use serenity::model::channel::{GuildChannel, Message};
use crate::util::ChannelParseMode::{FirstFullLastInitial, FullName};

use crate::website_builder::{GalleryInfo, PhotoInfo, save_thumbnail};

pub fn parse_photo_infos_from_message(message: Message) -> Vec<PhotoInfo> {
    message.attachments
        .into_iter()
        .filter(|attachment| { // Filter the attachments that are images
            attachment.content_type.is_some() && attachment.content_type.as_ref().unwrap().starts_with("image")
        })
        .map(|attachment| {
            let url = attachment.proxy_url.into_boxed_str();
            let thumbnail_url = save_thumbnail(&url);
            let picture_description =
                if message.content.is_empty() {
                    None
                } else {
                    Some(message.content.clone().into_boxed_str())
                };
            PhotoInfo {
                url,
                thumbnail_url,
                picture_description,
            }
        }).collect::<Vec<PhotoInfo>>()
}

pub async fn parse_gallery_info_from_channel(ctx: &Context, channel: &GuildChannel) -> Option<GalleryInfo> {
    let messages = channel.messages(&ctx.http, |message| message).await.unwrap();
    if messages.is_empty() { // If there are no messages return
        return None;
    }

    let author_text = messages.last().unwrap().author.tag();

    let picture_infos = messages
        .into_iter()
        .rev()
        .flat_map(|message| {
            parse_photo_infos_from_message(message)
        }).collect::<Vec<PhotoInfo>>();

    if picture_infos.is_empty() { // If there are no messages that have pictures in them return
        return None;
    }

    let author_name_channel = parse_author_name_from_channel_name(&channel.name, FirstFullLastInitial);

    let title = format!("{author_name_channel} ({author_text})").into_boxed_str();

    Some(
        GalleryInfo {
            title,
            picture_infos,
        }
    )
}

pub enum ChannelParseMode {
    FullName,
    FirstFullLastInitial,
}

pub fn parse_author_name_from_channel_name(channel_name: &str, channel_parse_mode: ChannelParseMode) -> String {
    match channel_parse_mode {
        FullName => {
            channel_name
                .split('-')
                .map(|s| {
                    let mut chars = s.chars();
                    let mut string = String::from(chars.next().unwrap().to_ascii_uppercase());
                    string += &*chars.as_str().to_ascii_lowercase();

                    string
                })
                .collect::<Vec<String>>()
                .join(" ")
        }
        FirstFullLastInitial => {
            let channel_name_parts = channel_name.split('-').collect::<Vec<&str>>();
            return if channel_name_parts.len() >= 2 {
                let mut first_name_chars = channel_name_parts[0].chars();
                let first_initial = first_name_chars.next().unwrap().to_ascii_uppercase();
                let rest_of_first_name = first_name_chars.as_str().to_ascii_lowercase();
                let last_initial = channel_name_parts[1].chars().next().unwrap().to_ascii_uppercase();
                format!("{}{} {}.", first_initial, rest_of_first_name, last_initial)
            } else { // If there isn't 2 parts to the name just return the channel name, this means someone didn't name their channel right
                channel_name.to_owned()
            };
        }
    }
}