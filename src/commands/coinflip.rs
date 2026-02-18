use anyhow::Result;
use serenity::all::{CommandInteraction, Context};

use crate::{DatabasePool, db, utils::make_resp};

pub async fn coinflip(ctx: &Context, command: CommandInteraction) -> Result<()> {
	let pool = ctx.data.read().await;
	let pool = pool.get::<DatabasePool>().unwrap();

	let uid = command.user.id.to_string();
	let user_elo = db::get_elo(pool, &uid).await?;

	let wager = command
		.data
		.options
		.first()
		.map(|x| x.value.as_i64().unwrap())
		.unwrap_or(user_elo / 2);

	if wager <= 0 {
		command
			.create_response(&ctx, make_resp(&format!("You can't wager {wager} elo")))
			.await?;

		return Ok(());
	} else if wager > user_elo {
		command
			.create_response(&ctx, make_resp("You can't wager more elo than you have"))
			.await?;

		return Ok(());
	}

	let won: bool = rand::random();

	if won {
		let gain = (wager as f64 * 0.8).ceil() as i64;
		db::set_elo(pool, &uid, user_elo + gain).await?;

		command
			.create_response(
				&ctx,
				make_resp(&format!(
					"You won {gain} elo! You now have {} elo.",
					user_elo + gain
				)),
			)
			.await?;
	} else {
		db::set_elo(pool, &uid, user_elo - wager).await?;

		command
			.create_response(
				&ctx,
				make_resp(&format!(
					"You lost {wager} elo. You now have {} elo.",
					user_elo - wager
				)),
			)
			.await?;
	}

	Ok(())
}
