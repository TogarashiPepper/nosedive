mod db;

use std::{env, time::Duration};

use serenity::{
    Client,
    all::{
        Context, CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, CreatePoll, CreatePollAnswer, EventHandler,
        GatewayIntents, Interaction,
    },
    async_trait,
    prelude::TypeMapKey,
};
use sqlx::SqlitePool;

#[tokio::main]
async fn main() {
    dotenvy::dotenv();

    let token = env::var("DISCORD_TOKEN").unwrap();
    let dbpool = SqlitePool::connect("sqlite:database.db").await.unwrap();
    let mut client = Client::builder(token, GatewayIntents::non_privileged())
        .event_handler(Handler)
        .await
        .unwrap();

    {
        let mut data = client.data.write().await;
        data.insert::<DatabasePool>(dbpool);
    }

    client.start().await.unwrap();
}

struct Handler;

struct DatabasePool;

impl TypeMapKey for DatabasePool {
    type Value = SqlitePool;
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(command) = interaction else {
            return;
        };

        let data = ctx.data.read().await;
        let dbpool = data.get::<DatabasePool>().unwrap();

        if command.data.name == "getelo" {
            let user = command.data.options[0]
                .value
                .as_user_id()
                .unwrap()
                .to_user(&ctx)
                .await
                .unwrap();

            let elo = db::get_elo(dbpool, &user.name).await.unwrap();
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

            return;
        } else if command.data.name != "createpoll" {
            return;
        }

        let user1 = command.data.options[0]
            .value
            .as_user_id()
            .unwrap()
            .to_user(&ctx)
            .await
            .unwrap();
        let user2 = command.data.options[1]
            .value
            .as_user_id()
            .unwrap()
            .to_user(&ctx)
            .await
            .unwrap();

        db::create_if_user(dbpool, &user1.name).await.unwrap();
        db::create_if_user(dbpool, &user2.name).await.unwrap();

        let poll = CreatePoll::new()
            .question("Which user is better? (poll ends in 1 minute)")
            .answers(vec![
                CreatePollAnswer::new().text(user1.name),
                CreatePollAnswer::new().text(user2.name),
            ])
            .duration(Duration::from_mins(60));

        let builder =
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().poll(poll));

        command.create_response(&ctx, builder).await.unwrap();

        tokio::time::sleep(Duration::from_mins(1)).await;

        let message = command.get_response(&ctx).await.unwrap();
        message.end_poll(&ctx).await.unwrap();

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
                    CreateInteractionResponseFollowup::new()
                        .content(format!("{loser} is a fat fucking chud, -{l_delta} elo. {winner} is a chad, +{w_delta} elo")),
                )
                .await
                .unwrap();
        }
    }
}
