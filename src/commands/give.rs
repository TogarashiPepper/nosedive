use anyhow::Result;
use serenity::all::{CommandInteraction, Context};

use crate::{
	DatabasePool, db,
	utils::{get_usr, make_resp},
};

pub async fn give(ctx: &Context, command: CommandInteraction) -> Result<()> {
	let pool = ctx.data.read().await;
	let pool = pool.get::<DatabasePool>().unwrap();

	let target = get_usr(ctx, &command.data.options[0].value).await;

	let uid = command.user.id.to_string();
	let tid = target.id.to_string();

	let user_elo = db::get_elo(pool, &uid).await?;
	let target_elo = db::get_elo(pool, &tid).await?;

	let amount = command
		.data
		.options
		.get(1)
		.map(|x| x.value.as_i64().unwrap())
		.unwrap_or(user_elo / 4);

	if amount <= 0 {
		command
			.create_response(
				&ctx,
				make_resp(&format!("You can't give someone {amount} elo")),
			)
			.await?;

		return Ok(());
	} else if amount > user_elo {
		command
			.create_response(
				&ctx,
				make_resp("You can't give someone more elo than you have"),
			)
			.await?;

		return Ok(());
	} else if target.bot {
		command
			.create_response(&ctx, make_resp("You can't give elo to a bot"))
			.await?;

		return Ok(());
	} else if tid == uid {
		command
			.create_response(&ctx, make_resp("You can't give elo to a yourself"))
			.await?;

		return Ok(());
	}

	db::set_elo(pool, &uid, user_elo - amount).await?;
	db::set_elo(pool, &tid, target_elo + amount).await?;

	command
		.create_response(
			&ctx,
			make_resp(&format!(
				"Successfully transferred {amount} elo to {target}."
			)),
		)
		.await?;

	Ok(())
}
