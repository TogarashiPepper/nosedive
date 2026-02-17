use std::fmt::Write;

use serenity::all::{
	CommandInteraction, Context, CreateInteractionResponse,
	CreateInteractionResponseMessage,
};
use sqlx::SqlitePool;

pub async fn leaderboard(
	dbpool: &SqlitePool,
	ctx: &Context,
	command: CommandInteraction,
) {
	let rankings = crate::db::rankings(dbpool).await.unwrap();
	let mut buf = String::from("Social Credit Leaderboard:\n");

	for (idx, rank) in rankings.into_iter().enumerate() {
		let _ = writeln!(buf, "{}. {}: {} elo", idx + 1, rank.0, rank.1);
	}

	command
		.create_response(
			&ctx,
			CreateInteractionResponse::Message(
				CreateInteractionResponseMessage::new().content(buf),
			),
		)
		.await
		.unwrap();
}
