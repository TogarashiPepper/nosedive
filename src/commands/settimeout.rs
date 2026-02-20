use anyhow::Result;
use serenity::all::{CommandInteraction, Context};

use crate::CooldownMap;
use crate::utils::make_resp;

pub async fn set_timeout(ctx: &Context, command: CommandInteraction) -> Result<()> {
	let timeout = command.data.options[0].value.as_i64().unwrap() as u64;

	let data = ctx
		.data
		.read()
		.await
		.get::<CooldownMap>()
		.unwrap()
		.data
		.clone();

	ctx.data
		.write()
		.await
		.insert::<CooldownMap>(CooldownMap { data, timeout });

	command
		.create_response(&ctx, make_resp(&format!("Timeout set to {timeout}.")))
		.await?;

	Ok(())
}
