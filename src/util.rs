


use serenity::client::Context;
use serenity::model::channel::{GuildChannel, Message};

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
    if messages.is_empty() {
        return None;
    }

    let author_text = messages.last().unwrap().author.tag();

    let picture_infos = messages
        .into_iter()
        .rev()
        .flat_map(|message| {
            parse_photo_infos_from_message(message)
        }).collect::<Vec<PhotoInfo>>();

    let author_name_channel = channel.name
        .split('-')
        .map(|s| {
            let mut chars = s.chars();
            let mut string = String::from(chars.next().unwrap().to_ascii_uppercase());
            string += chars.as_str();

            string
        })
        .collect::<Vec<String>>()
        .join(" ");

    let title = format!("{author_name_channel} ({author_text})").into_boxed_str();

    Some(
        GalleryInfo {
            title,
            picture_infos,
        }
    )
}