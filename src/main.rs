mod commands;
mod db;
mod utils;

use std::env;
use std::str::FromStr;

use serenity::all::{
	ChannelId, Context, EventHandler, GatewayIntents, Interaction, Permissions,
};
use serenity::prelude::TypeMapKey;
use serenity::{Client, async_trait};
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;

use crate::utils::make_resp;

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
				id          VARCHAR PRIMARY KEY NOT NULL,
				elo         FLOAT             NOT NULL,
				deviation   FLOAT             NOT NULL,
				volatility  FLOAT             NOT NULL,
				bytecoins   INTEGER           NOT NULL
			);

			CREATE TABLE IF NOT EXISTS coins
			(
				id          VARCHAR PRIMARY KEY NOT NULL,
				name        VARCHAR UNIQUE      NOT NULL,
				price       INTEGER             NOT NULL,
				total       INTEGER             NOT NULL
			);

			INSERT INTO coins (id, name, price, total) VALUES (1, 'ByteCoin', 100, 2500) ON CONFLICT(name) DO NOTHING;
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

pub struct Current;
pub struct DatabasePool;

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

		if command.guild_id.is_none() {
			command
				.create_response(&ctx, make_resp("You can't use Nosedive in DMs."))
				.await
				.unwrap();

			return;
		}

		match command.data.name.as_str() {
            "getelo" => commands::get_elo(&ctx, command).await.unwrap(),
            "leaderboard" => commands::leaderboard(&ctx, command).await.unwrap(),
            "challenge" => {
				if &command.channel.as_ref().unwrap().id != ctx.data.read().await.get::<Current>().unwrap() {
					let err = format!("Nosedive can't listen for polls in this channel, try in <#{}> instead.", ctx.data.read().await.get::<Current>().unwrap());

					command
						.create_response(
							&ctx,
							make_resp(&err)
						)
						.await
						.unwrap();
				}
				else {
					commands::challenge(&ctx, command).await.unwrap();
				}
			},
			"setchannel" => {
				if !command.member.as_ref().unwrap().permissions.unwrap().contains(Permissions::MANAGE_CHANNELS) {
					command
						.create_response(
							&ctx,
							make_resp("You need the `Manage Channels` permission to use `/setchannel`.")
						)
						.await
						.unwrap();
				}
				else {
					commands::set_channel(&ctx, command).await.unwrap();
				}
			},
			"bytecoin" => commands::bytecoin(&ctx, command).await.unwrap(),

			// This really shouldn't ever happen
            _ => command
                .create_response(
                    &ctx,
					make_resp("That command can't be handled by this version of nosedive. Please try updating or contacting the admin of this instance.")
                )
                .await
                .unwrap(),
        }
	}
}
