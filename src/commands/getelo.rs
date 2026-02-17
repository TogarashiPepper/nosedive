use serenity::all::{CommandInteraction, Context};

use crate::DatabasePool;
use crate::utils::make_resp;

pub async fn get_elo(ctx: &Context, command: CommandInteraction) {
	let data = ctx.data.write().await;
	let dbpool = data.get::<DatabasePool>().unwrap();

	let opt = command.data.options.first();

	let user = match opt
		.map(async |o| o.value.as_user_id().unwrap().to_user(&ctx).await.unwrap())
	{
		Some(usr) => &usr.await.name,
		None => &command.user.name,
	};

	let elo = crate::db::get_elo(dbpool, user).await.unwrap();
	command
		.create_response(&ctx, make_resp(&format!("User {} has {elo} elo", user)))
		.await
		.unwrap();
}
