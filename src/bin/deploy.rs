use std::env;

use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption, HttpBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let _ = dotenvy::dotenv();

	let application_id = env::var("DISCORD_APPLICATION_ID")?.parse()?;
	let token = env::var("DISCORD_TOKEN")?;
	let http = HttpBuilder::new(token)
		.application_id(application_id)
		.build();

	let user1 = CreateCommandOption::new(
		CommandOptionType::User,
		"first",
		"The first user in the match",
	)
	.required(true);
	let user2 = CreateCommandOption::new(
		CommandOptionType::User,
		"second",
		"The second user in the match",
	)
	.required(true);

	let createpoll = CreateCommand::new("createpoll")
		.description(
			"Pit two users against each other. Winner recieves socal credit (and loser loses it).",
		)
		.set_options(vec![user1, user2]);

	let user = CreateCommandOption::new(
		CommandOptionType::User,
		"user",
		"The user to fetch the elo of",
	)
	.required(true);

	let get_elo = CreateCommand::new("getelo")
		.description("Fetches the elo of a given user")
		.set_options(vec![user]);

	let leaderboard =
		CreateCommand::new("leaderboard").description("Display the global leaderbaord");

	let set_channel = CreateCommand::new("setchannel")
		.description("Sets which channel the bot will listen in (to stop users from farming in a channel people ignore).")
		.add_option(CreateCommandOption::new(CommandOptionType::Channel, "channel", "the channel to listen in").required(true));

	http.create_global_commands(&[createpoll, get_elo, leaderboard, set_channel])
		.await?;

	Ok(())
}
