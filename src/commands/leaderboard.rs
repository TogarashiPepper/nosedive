use std::fmt::Write;

use anyhow::Result;
use serenity::all::{CommandInteraction, Context};

use crate::DatabasePool;
use crate::utils::make_resp;

pub async fn leaderboard(ctx: &Context, command: CommandInteraction) -> Result<()> {
	let data = ctx.data.write().await;
	let dbpool = data.get::<DatabasePool>().unwrap();

	let rankings = crate::db::rankings(dbpool).await?;
	let mut buf = String::from("Social Credit Leaderboard:\n");

	for (idx, rank) in rankings.into_iter().enumerate() {
		let _ = writeln!(buf, "{}. {}: {} elo", idx + 1, rank.0, rank.1);
	}

	command.create_response(&ctx, make_resp(&buf)).await?;

	Ok(())
}
