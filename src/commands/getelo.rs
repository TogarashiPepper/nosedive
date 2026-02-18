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

	let elo = crate::db::get_elo(dbpool, &user.id.to_string())
		.await?
		.floor();
	command
		.create_response(&ctx, make_resp(&format!("User {} has {elo} elo.", user)))
		.await?;

	Ok(())
}
