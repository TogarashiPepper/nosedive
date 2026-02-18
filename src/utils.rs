use serenity::all::{
	CreateAllowedMentions, CreateInteractionResponse, CreateInteractionResponseFollowup,
	CreateInteractionResponseMessage,
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
