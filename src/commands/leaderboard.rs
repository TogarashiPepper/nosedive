use std::fmt::Write;

use anyhow::Result;
use serenity::all::{CommandInteraction, Context, UserId};

use crate::DatabasePool;
use crate::utils::make_resp;

pub async fn leaderboard(ctx: &Context, command: CommandInteraction) -> Result<()> {
	let data = ctx.data.read().await;
	let dbpool = data.get::<DatabasePool>().unwrap();

	let rankings = crate::db::rankings(dbpool).await?;
	let mut buf = String::from("Social Credit Leaderboard:\n");

	for (idx, rank) in rankings.into_iter().enumerate() {
		let user_id = UserId::new(rank.0.parse::<u64>()?);
		if rank.2 != 0.0 {
			writeln!(
				buf,
				"{}. <@{}>: {} elo ({} with bytecoins)",
				idx + 1,
				user_id,
				rank.1.floor(),
				(rank.1 + rank.2).floor() as i64
			)?;
		} else {
			writeln!(buf, "{}. <@{}>: {} elo", idx + 1, user_id, rank.1.floor())?;
		}
	}

	command.create_response(&ctx, make_resp(&buf)).await?;

	Ok(())
}
