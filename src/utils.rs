use serenity::all::{
	CommandDataOptionValue, Context, CreateAllowedMentions, CreateInteractionResponse,
	CreateInteractionResponseFollowup, CreateInteractionResponseMessage, User,
};

pub fn make_resp(content: &str) -> CreateInteractionResponse {
	CreateInteractionResponse::Message(
		CreateInteractionResponseMessage::new()
			.content(content)
			.allowed_mentions(CreateAllowedMentions::new().empty_users().empty_roles()),
	)
}

pub fn make_followup(content: &str) -> CreateInteractionResponseFollowup {
	CreateInteractionResponseFollowup::new()
		.content(content)
		.allowed_mentions(CreateAllowedMentions::new().empty_users().empty_roles())
}

pub async fn get_usr(ctx: &Context, option: &CommandDataOptionValue) -> User {
	option.as_user_id().unwrap().to_user(ctx).await.unwrap()
}
