use serenity::all::{
	CommandInteraction, Context, CreateInteractionResponse,
	CreateInteractionResponseMessage,
};

use crate::DatabasePool;

pub async fn get_elo(ctx: &Context, command: CommandInteraction) {
	let data = ctx.data.write().await;
	let dbpool = data.get::<DatabasePool>().unwrap();

	let user = command.data.options[0]
		.value
		.as_user_id()
		.unwrap()
		.to_user(&ctx)
		.await
		.unwrap();

	let elo = crate::db::get_elo(dbpool, &user.name).await.unwrap();
	command
		.create_response(
			&ctx,
			CreateInteractionResponse::Message(
				CreateInteractionResponseMessage::new()
					.content(format!("User {} has {elo} elo", user.name)),
			),
		)
		.await
		.unwrap();
}
