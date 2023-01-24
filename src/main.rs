use std::{env, thread};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use handlebars::{Handlebars, RenderError};
use std::string;
use ascii::IntoAsciiString;
use once_cell::sync::Lazy;
use serde::{Serialize, Serializer};

use serenity::{async_trait, Client};
use serenity::model::channel::{ChannelType, Message};
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::model::prelude::GuildChannel;
use serenity::prelude::{Context, EventHandler, GatewayIntents, TypeMapKey};
use tiny_http::{Header, Response, Server, StatusCode};
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
    author: String,
    picture_title: Option<String>,
}

#[derive(Serialize)]
struct HtmlPageData {
    pictures: String,
}


struct Handler;

impl Handler {
    async fn test(&self, ctx: &Context, messages: &[Message]) {
        let mut photo_infos: Vec<PhotoInfo> = Vec::new();
        messages.iter().for_each(|message| {
            let mut message_photo_infos = message.attachments.iter()
                .filter(|attachment| { // Filter the attachments that are images
                    attachment.content_type.is_some() && attachment.content_type.as_ref().unwrap().starts_with("image")
                })
                .map(|attachment| {
                    let url = attachment.proxy_url.clone();
                    let mut author = message.author.name.clone();
                    author.push_str(message.author.discriminator.to_string().as_str());
                    let picture_title = if message.content.is_empty() {
                        None
                    } else {
                        Some(message.content.clone())
                    };
                    PhotoInfo {
                        url,
                        author,
                        picture_title,
                    }
                }).collect();

            photo_infos.append(&mut message_photo_infos);
        });

        let mut picture_templates = String::new();
        photo_infos.iter().for_each(|photo_info| {
            let render_result = HANDLEBARS.render("picture_template", &photo_info);
            match render_result {
                Ok(rendered_result) => { picture_templates.push_str(&rendered_result) }
                Err(err) => { eprintln!("{}", err); }
            }
        });

        let built_html = HANDLEBARS.render("html_template", &HtmlPageData { pictures: picture_templates }).unwrap();

        {
            let data = ctx.data.read().await;
            let ampw = data.get::<ArcMutexPhotoWebserver>().unwrap();
            ampw.lock().unwrap().update_serving_page_src(built_html);
        }
        // let mut last = LAST_BUILT_WEBPAGE.lock().unwrap();
        // let _ = last.replace(built_html);
    }

    async fn get_connected_guilds() {}
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "/collectphotos" {
            if let Err(err) = msg.delete(&ctx.http).await {
                eprintln!("Error deleting message: {:?}", err);
            }
            let messages = msg.channel_id.messages(&ctx.http, |b| {
                b
            }).await.unwrap();
            let _ = messages.iter().for_each(|message| {
                eprintln!("Debug: message_data: {:?}", message);
            });

            self.test(&ctx, &messages).await;
        }


        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
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

        {
            let test_guild_id = guild_ids[0];
            let test_guild = ctx.http.get_guild(test_guild_id).await.unwrap();
            let tg_channels = test_guild.channels(&ctx.http).await.unwrap();
            let guild_channel_categories = tg_channels.iter().filter(|entry| {
                entry.1.kind == ChannelType::Category
            }).map(|entry| {
                entry.1
            }).collect::<Vec<&GuildChannel>>();


            guild_channel_categories.iter().for_each(|gc| {
                println!("Test channels: {}", gc.name);
            });
        }

        // Start the webserver once the discord bot has connected
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