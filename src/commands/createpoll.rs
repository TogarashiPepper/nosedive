use std::time::Duration;

use serenity::all::{
	CommandDataOptionValue, CommandInteraction, Context, CreateInteractionResponse,
	CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreatePoll,
	CreatePollAnswer, User,
};
use tokio::time;

use crate::{DatabasePool, db};

async fn get_usr(ctx: &Context, option: &CommandDataOptionValue) -> User {
	option.as_user_id().unwrap().to_user(ctx).await.unwrap()
}

pub async fn challenge(ctx: &Context, command: CommandInteraction) {
	let data = ctx.data.write().await;
	let dbpool = data.get::<DatabasePool>().unwrap();

	let user = &command.user;
	let target = get_usr(ctx, &command.data.options[0].value).await;

	if user == &target {
		command
			.create_response(
				&ctx,
				CreateInteractionResponse::Message(
					CreateInteractionResponseMessage::new()
						.content("You can't challenge yourself."),
				),
			)
			.await
			.unwrap();

		return;
	}

	db::create_if_user(dbpool, &user.name).await.unwrap();
	db::create_if_user(dbpool, &target.name).await.unwrap();

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

	command.create_response(&ctx, builder).await.unwrap();

	time::sleep(Duration::from_mins(1)).await;

	let message = command.get_response(&ctx).await.unwrap();
	message.end_poll(&ctx).await.unwrap();

	time::sleep(Duration::from_secs(4)).await;

	// Fetch it again after poll has ended, idk if this is necessary prob not
	let message = command.get_response(&ctx).await.unwrap();
	let msg_poll = message.poll.unwrap();
	let results = msg_poll.results.unwrap();

	let results_vec: Vec<(String, u64)> = results
		.answer_counts
		.iter()
		.filter_map(|answer_count| {
			// Find the answer text that matches this ID
			msg_poll
				.answers
				.iter()
				.find(|a| a.answer_id == answer_count.id)
				.and_then(|a| a.poll_media.text.clone())
				.map(|text| (text, answer_count.count))
		})
		.collect();

	let (winner, w_scr) = results_vec.iter().max_by_key(|r| r.1).unwrap();
	let (loser, l_scr) = results_vec.iter().min_by_key(|r| r.1).unwrap();

	if w_scr == l_scr {
		command
			.create_followup(
				&ctx,
				CreateInteractionResponseFollowup::new()
					.content(format!("Votes are tied. {winner} and {loser} tied.")),
			)
			.await
			.unwrap();
	} else {
		let (w_delta, l_delta) = db::finalize_match(dbpool, winner, loser).await.unwrap();
		let l_delta = l_delta.abs();

		command
			.create_followup(
				&ctx,
				CreateInteractionResponseFollowup::new().content(format!(
					"{loser} is a fat fucking chud, -{l_delta} elo. {winner} is a chad, +{w_delta} elo"
				)),
			)
			.await
			.unwrap();
	}
}
