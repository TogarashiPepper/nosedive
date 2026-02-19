use anyhow::Result;
use serenity::all::{CommandDataOptionValue, CommandInteraction, Context};

use crate::utils::make_resp;
use crate::{DatabasePool, db};

pub async fn bytecoin(ctx: &Context, command: CommandInteraction) -> Result<()> {
	let pool = ctx.data.read().await;
	let pool = pool.get::<DatabasePool>().unwrap();

	// Subcommand will always be present
	let Some(subcommand) = command.data.options.get(0) else {
		unreachable!()
	};

	match subcommand.name.as_ref() {
		"price" => {
			let number = db::get_bytecoin_total(pool).await?;

			// INITIAL_BASE_PRICE: 100
			// TOTAL_COINS: 2500
			let price = 100 + 25 * (2500 - number);

			command
				.create_response(
					&ctx,
					make_resp(&format!("Price of next bytecoin: {price} elo.")),
				)
				.await?;
		}
		"buy" => {
			let total = db::get_bytecoin_total(pool).await?;
			let uid = command.user.id.to_string();
			let user_bytecoins = db::get_user_bytecoins(pool, &uid).await?;

			// Required option
			let CommandDataOptionValue::SubCommand(options) = &subcommand.value else {
				unreachable!()
			};

			// Required option
			let number = options.get(0).map(|x| x.value.as_i64().unwrap()).unwrap();

			if number <= 0 {
				command
					.create_response(
						&ctx,
						make_resp(&format!("You must buy positive number of bytecoins.")),
					)
					.await?;

				return Ok(());
			}

			// INITIAL_BASE_PRICE: 100
			// TOTAL_COINS: 2500
			// FORMULA: (BASE_PRICE + 25(TOTAL - CURRENT))n + 25n(n-1)/2

			let cost =
				(100 + 25 * (2500 - total)) * number + 25 * number * (number - 1) / 2;

			let elo = db::get_elo(pool, &uid).await?;

			if cost > elo.floor() as i64 {
				command
					.create_response(
						&ctx,
						make_resp(&format!(
							"You don't have money to purchase that many bytecoins (<{}).",
							cost - elo.floor() as i64,
						)),
					)
					.await?;

				return Ok(());
			}

			let new_elo = elo - cost as f64;

			db::set_elo(pool, &uid, new_elo).await?;
			db::set_user_bytecoins(pool, &uid, user_bytecoins + number).await?;
			db::set_bytecoin_total(pool, total - number).await?;

			command
				.create_response(
					&ctx,
					make_resp(&format!(
						"You bought {number} bytecoins for {} elo!",
						(elo - new_elo).floor()
					)),
				)
				.await?;
		}
		"held" => {
			let uid = command.user.id.to_string();
			let number = db::get_user_bytecoins(pool, &uid).await?;

			command
				.create_response(
					&ctx,
					make_resp(&format!("You have {number} bytecoins!",)),
				)
				.await?;
		}
		"sell" => {
			let total = db::get_bytecoin_total(pool).await?;
			let uid = command.user.id.to_string();
			let user_bytecoins = db::get_user_bytecoins(pool, &uid).await?;

			// Required option
			let CommandDataOptionValue::SubCommand(options) = &subcommand.value else {
				unreachable!()
			};

			// Required option
			let number = options.get(0).map(|x| x.value.as_i64().unwrap()).unwrap();

			if number <= 0 {
				command
					.create_response(
						&ctx,
						make_resp(&format!(
							"You must sell positive number of bytecoins."
						)),
					)
					.await?;

				return Ok(());
			} else if number > user_bytecoins {
				command
					.create_response(
						&ctx,
						make_resp(&format!(
							"You can't sell more number of bytecoins than you have ({user_bytecoins})."
						)),
					)
					.await?;

				return Ok(());
			}

			// INITIAL_BASE_PRICE: 100
			// TOTAL_COINS: 2500
			// FORMULA: (BASE_PRICE + 25(TOTAL - CURRENT))n - 25n(n-1)/2
			// 1 is subtracted from 2500 to get the price of last bitcoin bought
			let gain =
				(100 + 25 * (2500 - total - 1)) * number - 25 * number * (number - 1) / 2;

			let elo = db::get_elo(pool, &uid).await?;

			let new_elo = elo + gain as f64;

			db::set_elo(pool, &uid, new_elo).await?;
			db::set_user_bytecoins(pool, &uid, user_bytecoins - number).await?;
			db::set_bytecoin_total(pool, total + number).await?;

			command
				.create_response(
					&ctx,
					make_resp(&format!(
						"You sold {number} bytecoins and gained {} elo!",
						(new_elo - elo).floor()
					)),
				)
				.await?;
		}

		// All cases have been covered above
		_ => unreachable!(),
	}
	Ok(())
}
