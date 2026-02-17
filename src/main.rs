mod commands;
mod db;

use std::env;
use std::str::FromStr;

use serenity::all::{
	ChannelId, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
	EventHandler, GatewayIntents, Interaction, Permissions,
};
use serenity::prelude::TypeMapKey;
use serenity::{Client, async_trait};
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;

#[tokio::main]
async fn main() {
	let _ = dotenvy::dotenv();

	let token = env::var("DISCORD_TOKEN").unwrap();

	let dbopts = SqliteConnectOptions::from_str(&env::var("DATABASE_URL").unwrap())
		.unwrap()
		.create_if_missing(true);
	let dbpool = SqlitePool::connect_with(dbopts).await.unwrap();

	sqlx::query!(
		r#"
			CREATE TABLE IF NOT EXISTS users
			(
				username    VARCHAR PRIMARY KEY NOT NULL,
				elo         INTEGER             NOT NULL
			);
		"#
	)
	.execute(&dbpool)
	.await
	.unwrap();

	let mut client = Client::builder(token, GatewayIntents::non_privileged())
		.event_handler(Handler)
		.await
		.unwrap();

	{
		let mut data = client.data.write().await;
		data.insert::<DatabasePool>(dbpool);
		data.insert::<Current>(ChannelId::new(806571996485386240));
	}

	client.start().await.unwrap();
}

struct Handler;

struct Current;
struct DatabasePool;

impl TypeMapKey for DatabasePool {
	type Value = SqlitePool;
}

impl TypeMapKey for Current {
	type Value = ChannelId;
}

#[async_trait]
impl EventHandler for Handler {
	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		let Interaction::Command(command) = interaction else {
			return;
		};

		match command.data.name.as_str() {
            "getelo" => commands::get_elo(&ctx, command).await,
            "leaderboard" => commands::leaderboard(&ctx, command).await,
            "challenge" => {
				if &command.channel.as_ref().unwrap().id != ctx.data.read().await.get::<Current>().unwrap() {
					command
						.create_response(
							&ctx,
							CreateInteractionResponse::Message(
								CreateInteractionResponseMessage::new()
									.content(format!("Nosedive can't listen for polls in this channel, try in <#{}> instead", command.channel.as_ref().unwrap().id))
							)
						)
						.await
						.unwrap();
				}
				else {
					commands::challenge(&ctx, command).await;
				}
			},
			"setchannel" => {
				if !command.member.as_ref().unwrap().permissions.unwrap().contains(Permissions::MANAGE_CHANNELS) {
					command
						.create_response(
							&ctx,
							CreateInteractionResponse::Message(
								CreateInteractionResponseMessage::new()
									.content("You need the Manage Channels permission to use /setchannel")
							)
						)
						.await
						.unwrap();
				}
				else {
					commands::set_channel(&ctx, command).await;
				}
			},

			// This really shouldn't ever happen
            _ => command
                .create_response(
                    &ctx,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("That command can't be handled by this version of nosedive. Please try updating or contacting the admin of this instance."),
                    ),
                )
                .await
                .unwrap(),
        }
	}
}
