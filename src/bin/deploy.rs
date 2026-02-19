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
		"target",
		"The user you wish to challenge.",
	)
	.required(true);

	let challenge = CreateCommand::new("challenge")
		.description("Pit yourself against someone. Winner gets social credit.")
		.set_options(vec![user1]);

	let user = CreateCommandOption::new(
		CommandOptionType::User,
		"user",
		"The user whose elo you wish to see.",
	);

	let get_elo = CreateCommand::new("getelo")
		.description("Fetches the elo of a given user.")
		.set_options(vec![user]);

	let leaderboard =
		CreateCommand::new("leaderboard").description("Display the global leaderboard.");

	let set_channel = CreateCommand::new("setchannel")
		.description("Sets which channel the bot will listen in to stop users from farming in a channel people ignore.")
		.add_option(CreateCommandOption::new(CommandOptionType::Channel, "channel", "The channel in which challenges will be allowed.").required(true));

	let gift = CreateCommand::new("give")
		.description("Give elo to another user.")
		.add_option(
			CreateCommandOption::new(
				CommandOptionType::User,
				"recipient",
				"The user who is to recieve the elo.",
			)
			.required(true),
		)
		.add_option(CreateCommandOption::new(
			CommandOptionType::Integer,
			"amount",
			"The amount to gift (defaults to one-fourth elo contained).",
		));

	http.create_global_commands(&[
		challenge,
		get_elo,
		leaderboard,
		set_channel,
		gift,
	])
	.await?;

	Ok(())
}
