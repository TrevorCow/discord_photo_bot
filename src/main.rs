extern crate core;

use std::env;
use std::fmt::Display;
use std::io::Stdout;
use std::process::{Command, exit, Stdio};
use std::sync::{Arc};
use chrono::Local;

use serenity::{async_trait, Client};
use serenity::client::{Context, EventHandler};
use serenity::model::channel::{GuildChannel, Message};
use serenity::model::gateway::Ready;
use serenity::model::guild::Guild;
use serenity::prelude::{GatewayIntents, TypeMapKey};
use futures::{StreamExt};
use serenity::client::bridge::gateway::ShardManager;
use serenity::Error::Other;
use tokio::sync::Mutex;
use crate::util::{parse_gallery_info_from_channel};
use crate::website_builder::{build_website, clean_website_folder, PageBuildInfo, PageInfo};

mod util;
mod website_builder;

const BOT_GATEWAY_INTENTS: u64 = GatewayIntents::GUILD_MESSAGES.bits() |
    GatewayIntents::MESSAGE_CONTENT.bits() |
    GatewayIntents::DIRECT_MESSAGES.bits();

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct BotEventHandler;

impl BotEventHandler {
    async fn collect_photos(&self, ctx: &Context, msg: Message) {
        let collect_photos_result = async {
            let current_guild = ctx.http.get_guild(msg.guild_id.unwrap().0).await?;
            let guild_channels = current_guild.channels(&ctx.http).await?;

            let message_channel_id = msg.channel_id;
            let message_channel = match msg.channel(&ctx.http).await?.guild() {
                Some(guild_channel) => guild_channel,
                None => {
                    message_channel_id.say(&ctx.http, "This command must be run in a channel that belongs to a guild! (No private messages)").await.unwrap();
                    return Err(Other("Tried to run command `collectphotos` from a channel that belongs does not belong to a guild. (Eg. private message)"));
                }
            };
            let message_parent_category = match message_channel.parent_id {
                Some(message_parent_id) => message_parent_id.to_channel(&ctx.http).await?.category().unwrap(),
                None => {
                    message_channel_id.say(&ctx.http, "This command must be run in a channel that belongs to a category!").await?;
                    return Err(Other("Tried to run command `collectphotos` from a channel that does not belong to a category"));
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

            let galleries = futures::future::join_all(
                category_children.into_iter()
                    .map(|channel| async {
                        parse_gallery_info_from_channel(ctx, channel).await
                    })
            ).await
                .into_iter()
                .flatten()
                .collect();


            let page_title = format!("{} Photo Galleries", current_guild.name).into_boxed_str();
            let page_build_info = PageBuildInfo {
                guild_built_from: current_guild.name.into_boxed_str(),
                channel_built_from: message_channel.name.into_boxed_str(),
                user_built_by: msg.author.tag().into_boxed_str(),
                built_time: Local::now().to_string().into_boxed_str(),
            };
            
            let page_info = PageInfo {
                page_title,
                page_build_info,
                galleries,
            };

            build_website(page_info);
            Ok(())
        }.await;
        match collect_photos_result {
            Ok(_) => {}
            Err(err) => eprintln!("Error collecting photos: {err}"),
        }
    }
}

#[async_trait]
impl EventHandler for BotEventHandler {
    async fn guild_create(&self, _ctx: Context, guild: Guild) {
        println!("\tConnected to guild: {}", guild.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "/collectphotos" {
            if let Err(err) = msg.delete(&ctx.http).await {
                eprintln!("Error deleting message: {err:?}");
            }
            self.collect_photos(&ctx, msg).await;
        } else if msg.content == "/leave" {
            if let Err(why) = ctx.http.leave_guild(msg.guild_id.unwrap().0).await {
                println!("Error leaving guild: {why:?}");
            }
        } else if msg.content == "/ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
    }


    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Bot `{}` has started and is connected to {} guilds!", ready.user.name, ready.guilds.len());

        futures::stream::iter(ready.guilds.iter())
            .for_each(|possible_guild| async {
                let partial_guild = match ctx.http.get_guild(possible_guild.id.0).await {
                    Ok(pg) => { pg }
                    Err(err) => {
                        eprintln!("Error getting guild with id `{}`: {}", possible_guild.id.0, err);
                        return;
                    }
                };
                println!("\tConnected to guild: {}", partial_guild.name);
            }).await;
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(&token, GatewayIntents::from_bits_truncate(BOT_GATEWAY_INTENTS))
        .event_handler(BotEventHandler)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    if let Err(err) = client.start().await {
        println!("Client error: {err:?}");
    }
}
