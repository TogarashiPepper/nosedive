use serenity::all::{
	CreateInteractionResponse, CreateInteractionResponseFollowup,
	CreateInteractionResponseMessage,
};

pub fn make_resp(content: &str) -> CreateInteractionResponse {
	CreateInteractionResponse::Message(
		CreateInteractionResponseMessage::new().content(content),
	)
}

pub fn make_followup(content: &str) -> CreateInteractionResponseFollowup {
	CreateInteractionResponseFollowup::new().content(content)
}
