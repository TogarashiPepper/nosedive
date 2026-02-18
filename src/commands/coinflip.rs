use anyhow::Result;
use serenity::all::{CommandInteraction, Context};

use crate::utils::make_resp;
use crate::{DatabasePool, db};

pub async fn coinflip(ctx: &Context, command: CommandInteraction) -> Result<()> {
	let pool = ctx.data.read().await;
	let pool = pool.get::<DatabasePool>().unwrap();

	let uid = command.user.id.to_string();
	let user_elo = db::get_elo(pool, &uid).await?;

	dbg!(command.data.options.first());

	let wager = command
		.data
		.options
		.first()
		.map(|x| x.value.as_i64().unwrap() as f64)
		.unwrap_or(user_elo / 2.0);

	if wager <= 0.0 {
		command
			.create_response(&ctx, make_resp(&format!("You can't wager negative elo.")))
			.await?;

		return Ok(());
	} else if wager > user_elo {
		command
			.create_response(&ctx, make_resp("You can't wager more elo than you have."))
			.await?;

		return Ok(());
	}

	let won: bool = rand::random();

	if won {
		let gain = (wager * 0.2).floor();
		db::set_elo(pool, &uid, user_elo + gain).await?;

		command
			.create_response(
				&ctx,
				make_resp(&format!(
					"You won {gain} elo! You now have {} elo.",
					(user_elo + gain).floor()
				)),
			)
			.await?;
	} else {
		db::set_elo(pool, &uid, user_elo - wager).await?;

		command
			.create_response(
				&ctx,
				make_resp(&format!(
					"You lost {} elo. You now have {} elo.",
					wager.floor(),
					(user_elo - wager).floor()
				)),
			)
			.await?;
	}

	Ok(())
}
