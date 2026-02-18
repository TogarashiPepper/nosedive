use std::time::Duration;

use anyhow::Result;
use serenity::all::{
	CommandDataOptionValue, CommandInteraction, Context, CreateInteractionResponse,
	CreateInteractionResponseMessage, CreatePoll, CreatePollAnswer, User,
};
use tokio::time;

use crate::utils::{make_followup, make_resp};
use crate::{DatabasePool, db};

async fn get_usr(ctx: &Context, option: &CommandDataOptionValue) -> User {
	option.as_user_id().unwrap().to_user(ctx).await.unwrap()
}

pub async fn challenge(ctx: &Context, command: CommandInteraction) -> Result<()> {
	let data = ctx.data.write().await;
	let dbpool = data.get::<DatabasePool>().unwrap();

	let user = &command.user;
	let target = get_usr(ctx, &command.data.options[0].value).await;

	if user == &target {
		command
			.create_response(&ctx, make_resp("You can't challenge yourself"))
			.await?;

		return Ok(());
	} else if target.bot {
		command
			.create_response(&ctx, make_resp("You can't challenge a bot"))
			.await?;

		return Ok(());
	}

	db::create_if_user(dbpool, &user.name).await?;
	db::create_if_user(dbpool, &target.name).await?;

	let poll = CreatePoll::new()
		.question(
			"Which user is more morally or comedically superior here? (poll ends in 1 minute)",
		)
		.answers(vec![
			CreatePollAnswer::new().text(&user.name),
			CreatePollAnswer::new().text(target.name),
		])
		.duration(Duration::from_mins(60));

	let builder = CreateInteractionResponse::Message(
		CreateInteractionResponseMessage::new().poll(poll),
	);

	command.create_response(&ctx, builder).await?;

	time::sleep(Duration::from_mins(1)).await;

	let message = command.get_response(&ctx).await?.end_poll(&ctx).await?;

	// We know this message has a poll, and results (since we just made and ended it)
	let poll = message.poll.unwrap();
	let results = poll.results.unwrap();

	let results: Vec<(String, u64)> = results
		.answer_counts
		.iter()
		.filter_map(|answer_count| {
			// Find the answer text that matches this ID
			poll.answers
				.iter()
				.find(|a| a.answer_id == answer_count.id)
				.and_then(|a| a.poll_media.text.clone())
				.map(|text| (text, answer_count.count))
		})
		.collect();

	// By definition this poll has at least 2 elements, unwrap is fine
	let (winner, w_scr) = results.iter().max_by_key(|r| r.1).unwrap();
	let (loser, l_scr) = results.iter().min_by_key(|r| r.1).unwrap();

	let res = if w_scr == l_scr {
		format!("{winner} and {loser} tied. No elo has been lost or gained")
	} else {
		let (w_delta, l_delta) = db::finalize_match(dbpool, winner, loser).await?;
		let l_delta = l_delta.abs();
		format!(
			"{loser} has lost (-{l_delta} elo). {winner} is the morally or comedically superior individual (+{w_delta} elo)"
		)
	};

	command.create_followup(&ctx, make_followup(&res)).await?;

	Ok(())
}
