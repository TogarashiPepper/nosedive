use serenity::all::{
	CommandInteraction, Context, CreateInteractionResponse,
	CreateInteractionResponseMessage,
};

use crate::Current;

pub async fn set_channel(ctx: &Context, command: CommandInteraction) {
	let channel = command.data.options[0]
		.value
		.as_channel_id()
		.unwrap()
		.to_channel(ctx)
		.await
		.unwrap();

	ctx.data.write().await.insert::<Current>(channel.id());

	command
		.create_response(
			&ctx,
			CreateInteractionResponse::Message(
				CreateInteractionResponseMessage::new().content(format!(
					"Nosedive will now only listen for polls in {}",
					channel
				)),
			),
		)
		.await
		.unwrap();
}
