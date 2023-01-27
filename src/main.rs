use std::{env, fs};
use std::sync::{Arc, Mutex};
use chrono::Local;

use handlebars::Handlebars;
use once_cell::sync::Lazy;
use serde::Serialize;
use serenity::{async_trait, Client};
use serenity::futures::future::join_all;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::GuildChannel;
use serenity::prelude::{Context, EventHandler, GatewayIntents, TypeMapKey};

use crate::webserver::PhotoWebserver;

mod webserver;

const HTML_TEMPLATE: &str = include_str!("picture_template.html");

const PICTURE_TEMPLATE_NEW: &str = r#"
<div class="gallery_image">
    <img src="{{url}}" alt="9. THis iS pHOto!">
    <p>{{picture_title}}</p>
</div>
"#;


static HANDLEBARS: Lazy<Handlebars> = Lazy::new(|| {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("picture_template", PICTURE_TEMPLATE_NEW).expect("TODO: panic message");
    handlebars.register_template_string("html_template", HTML_TEMPLATE).expect("TODO: panic message");
    handlebars
});

#[derive(Serialize)]
struct PhotoInfo {
    url: String,
    picture_description: Option<String>,
}

#[derive(Serialize)]
struct GalleryInfo {
    title: String,
    picture_infos: Vec<PhotoInfo>,
}

#[derive(Serialize)]
struct PageInfo {
    page_title: String,
    page_build_info: String,
    galleries: Vec<GalleryInfo>,
}


struct Handler;

impl Handler {
    async fn cmd_collect_photos(&self, ctx: &Context, msg: Message) {
        let current_guild = ctx.http.get_guild(msg.guild_id.unwrap().0).await.unwrap();
        let guild_channels = current_guild.channels(&ctx.http).await.unwrap();

        let message_channel_id = msg.channel_id;
        let message_channel = match msg.channel(&ctx.http).await.unwrap().guild() {
            Some(guild_channel) => guild_channel,
            None => {
                message_channel_id.say(&ctx.http, "This command must be run in a channel that belongs to a guild! (No private messages)").await.unwrap();
                return;
            }
        };
        let message_parent_category = match message_channel.parent_id {
            Some(message_parent_id) => {
                message_parent_id.to_channel(&ctx.http).await.unwrap().category().unwrap()
            }
            None => {
                message_channel_id.say(&ctx.http, "This command must be run in a channel that belongs to a category!").await.unwrap();
                return;
            }
        };
        let category_children = guild_channels.iter()
            .filter(|entry| {
                match entry.1.parent_id {
                    Some(parent_id) => { parent_id == message_parent_category.id }
                    None => { false }
                }
            })
            .map(|entry| entry.1)
            .collect::<Vec<&GuildChannel>>();

        let maybe_galleries = join_all(
            category_children.iter()
                .map(|channel| async {
                    let (discord_author_text, picture_infos) = match self.collect_channel_photos_and_author(ctx, channel).await {
                        Some(s) => { s }
                        None => {
                            return None;
                        }
                    };

                    let author_name_channel = channel.name.clone()
                        .split('-')
                        .map(|s| {
                            let mut chars = s.chars();
                            let mut string = String::from(chars.next().unwrap().to_ascii_uppercase());
                            string += chars.as_str();

                            string
                        })
                        .collect::<Vec<String>>()
                        .join(" ");

                    let title = format!("{} ({})", author_name_channel, discord_author_text);

                    Some(
                        GalleryInfo {
                            title,
                            picture_infos,
                        }
                    )
                })
        ).await;

        let galleries = maybe_galleries.into_iter().flatten().collect();

        let page_build_info = format!("Page build from channel `{}` by `{}` on {}", message_channel.name, msg.author.tag(), Local::now());

        let page_info = PageInfo {
            page_title: format!("{} Photo Galleries", current_guild.name),
            page_build_info,
            galleries,
        };

        self.build_gallery_webpage(ctx, page_info).await;
    }

    async fn build_gallery_webpage(&self, ctx: &Context, page_info: PageInfo) {
        let built_html = HANDLEBARS.render("html_template", &page_info).unwrap();

        fs::write("last_built_page.html", &built_html).expect("Unable to write file");

        {
            let data = ctx.data.read().await;
            let ampw = data.get::<ArcMutexPhotoWebserver>().unwrap();
            ampw.lock().unwrap().update_serving_page_src(built_html);
        }
    }

    async fn collect_channel_photos_and_author(&self, ctx: &Context, channel: &GuildChannel) -> Option<(String, Vec<PhotoInfo>)> {
        let messages = channel.messages(&ctx.http, |message| message).await.unwrap();
        if messages.is_empty() {
            return None;
        }
        let photo_infos = messages
            .iter().rev()
            .flat_map(|message| {
                message.attachments.iter()
                    .filter(|attachment| { // Filter the attachments that are images
                        attachment.content_type.is_some() && attachment.content_type.as_ref().unwrap().starts_with("image")
                    })
                    .map(|attachment| {
                        let url = attachment.proxy_url.clone();
                        let picture_description =
                            if message.content.is_empty() {
                                None
                            } else {
                                Some(message.content.clone())
                            };
                        PhotoInfo {
                            url,
                            picture_description,
                        }
                    }).collect::<Vec<PhotoInfo>>()
            }).collect::<Vec<PhotoInfo>>();

        let first_message_discord_author = messages.last().unwrap().author.clone();
        // let author_text = format!("{}#{:0>4}", first_message_discord_author.name, first_message_discord_author.discriminator);
        let author_text = first_message_discord_author.tag();

        Some((author_text, photo_infos))
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "/collectphotos" {
            if let Err(err) = msg.delete(&ctx.http).await {
                eprintln!("Error deleting message: {:?}", err);
            }

            self.cmd_collect_photos(&ctx, msg).await;
        } else if msg.content == "/leave" {
            if let Err(why) = ctx.http.leave_guild(msg.guild_id.unwrap().0).await {
                println!("Error leaving guild: {:?}", why);
            }
        } else if msg.content == "/ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected ot {} guilds!", ready.user.name, ready.guilds.len());

        let guilds = ready.guilds;
        let guild_ids: Vec<u64> = guilds.iter().map(|guild| { guild.id.0 }).collect();
        for guild_id in &guild_ids {
            let guild_name = match ctx.http.get_guild(*guild_id).await {
                Ok(partial_guild) => { partial_guild.name }
                Err(err) => {
                    eprintln!("Error getting guild name for id `{}`: {}", guild_id, err);
                    return;
                }
            };
            println!("\tConnected to guild: {}", guild_name);
        }
    }
}

struct ArcMutexPhotoWebserver;

impl TypeMapKey for ArcMutexPhotoWebserver {
    type Value = Arc<Mutex<PhotoWebserver>>;
}

#[tokio::main]
async fn main() {
    let webserver = Arc::new(Mutex::new(PhotoWebserver::new()));
    {
        // Start the webserver
        let ws_lock = webserver.lock().unwrap();
        ws_lock.spawn_server(8080);
    }


    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ArcMutexPhotoWebserver>(webserver);
    }

    if let Err(err) = client.start().await {
        println!("Client error: {:?}", err);
    }
}