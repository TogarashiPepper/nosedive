use anyhow::Result;
use serenity::all::{CommandInteraction, Context};

use crate::DatabasePool;
use crate::utils::make_resp;

pub async fn get_elo(ctx: &Context, command: CommandInteraction) -> Result<()> {
	let data = ctx.data.read().await;
	let dbpool = data.get::<DatabasePool>().unwrap();

	let opt = command.data.options.first();

	let user = match opt.map(async |o| o.value.as_user_id().unwrap().to_user(&ctx).await)
	{
		Some(usr) => &usr.await?,
		None => &command.user,
	};

	if user.bot {
		command
			.create_response(&ctx, make_resp("Bots can't have elo, silly."))
			.await?;

		return Ok(());
	}

	let (elo, bc_worth) =
		crate::db::get_elo_with_bc(dbpool, &user.id.to_string()).await?;

	let res = if bc_worth != 0.0 {
		format!(
			"User {} has {} elo ({} with bytecoins).",
			user,
			elo.floor(),
			(elo + bc_worth).floor()
		)
	} else {
		format!("User {} has {} elo.", user, elo.floor())
	};

	command.create_response(&ctx, make_resp(&res)).await?;

	Ok(())
}
