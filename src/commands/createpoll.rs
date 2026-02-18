use std::time::Duration;

use anyhow::Result;
use serenity::all::{
	AnswerId, CommandDataOptionValue, CommandInteraction, Context,
	CreateInteractionResponse, CreateInteractionResponseMessage, CreatePoll,
	CreatePollAnswer, Poll, User,
};
use tokio::time;

use crate::utils::{make_followup, make_resp};
use crate::{DatabasePool, db};

async fn get_usr(ctx: &Context, option: &CommandDataOptionValue) -> User {
	option.as_user_id().unwrap().to_user(ctx).await.unwrap()
}

pub async fn challenge(ctx: &Context, command: CommandInteraction) -> Result<()> {
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

	// Create a scope to acquire and drop the lock in
	{
		let data = ctx.data.write().await;
		let dbpool = data.get::<DatabasePool>().unwrap();

		db::create_if_user(dbpool, &user.id.to_string()).await?;
		db::create_if_user(dbpool, &target.id.to_string()).await?;
	}

	let poll = CreatePoll::new()
		.question(
			"Which user is more morally or comedically superior here? (poll ends in 1 minute)",
		)
		.answers(vec![
			CreatePollAnswer::new().text(&user.name),
			CreatePollAnswer::new().text(&target.name),
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

	let user_answer_id = poll
		.answers
		.iter()
		.find_map(|pa| {
			(pa.poll_media.text.as_deref()? == user.name).then_some(pa.answer_id)
		})
		.unwrap();
	let target_answer_id = poll
		.answers
		.iter()
		.find_map(|pa| {
			(pa.poll_media.text.as_deref()? == target.name).then_some(pa.answer_id)
		})
		.unwrap();

	let user_score = results
		.answer_counts
		.iter()
		.find_map(|pac| (pac.id == user_answer_id).then_some(pac.count))
		.unwrap_or(0);
	let target_score = results
		.answer_counts
		.iter()
		.find_map(|pac| (pac.id == target_answer_id).then_some(pac.count))
		.unwrap_or(0);

	// No separate scope for the lock here since we exit soon anyways
	let data = ctx.data.write().await;
	let dbpool = data.get::<DatabasePool>().unwrap();

	let res = if user_score == target_score {
		format!("{user} and {target} tied. No elo has been lost or gained")
	} else {
		let loser: &User;
		let winner: &User;

		if user_score > target_score {
			winner = user;
			loser = &target;
		} else {
			winner = &target;
			loser = user;
		}

		let (w_delta, l_delta) =
			db::finalize_match(dbpool, &winner.id.to_string(), &loser.id.to_string())
				.await?;
		let l_delta = l_delta.abs();

		format!(
			"{loser} has lost (-{l_delta} elo). {winner} is the morally or comedically superior individual (+{w_delta} elo)"
		)
	};

	command.create_followup(&ctx, make_followup(&res)).await?;

	Ok(())
}
