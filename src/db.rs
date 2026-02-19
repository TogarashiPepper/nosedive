use std::time::Duration;

use instant_glicko_2::engine::{MatchResult, RatingEngine};
use instant_glicko_2::{Parameters, PublicRating};
use sqlx::SqlitePool;

pub async fn user_exists(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
	let exists: i64 =
		sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)", id)
			.fetch_one(pool)
			.await?;

	Ok(exists == 1)
}

pub async fn create_user(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
	sqlx::query!(
		r#"
			INSERT INTO users
			VALUES ($1, 1500.0, 350.0, 0.06)
		"#,
		id,
	)
	.execute(pool)
	.await?;

	Ok(())
}

pub async fn create_if_user(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
	if !user_exists(pool, id).await? {
		create_user(pool, id).await?;
	}

	Ok(())
}

pub async fn get_elo(pool: &SqlitePool, id: &str) -> Result<f64, sqlx::Error> {
	create_if_user(pool, id).await?;

	let elo: f64 = sqlx::query_scalar!("SELECT elo FROM users WHERE id = $1", id)
		.fetch_one(pool)
		.await?;

	Ok(elo)
}

pub async fn get_deviation(pool: &SqlitePool, id: &str) -> Result<f64, sqlx::Error> {
	create_if_user(pool, id).await?;

	let elo: f64 = sqlx::query_scalar!("SELECT deviation FROM users WHERE id = $1", id)
		.fetch_one(pool)
		.await?;

	Ok(elo)
}

pub async fn get_volatility(pool: &SqlitePool, id: &str) -> Result<f64, sqlx::Error> {
	create_if_user(pool, id).await?;

	let elo: f64 = sqlx::query_scalar!("SELECT volatility FROM users WHERE id = $1", id)
		.fetch_one(pool)
		.await?;

	Ok(elo)
}

pub async fn set_elo(pool: &SqlitePool, id: &str, elo: f64) -> Result<(), sqlx::Error> {
	sqlx::query_scalar!("UPDATE users SET elo = $1 WHERE id = $2", elo, id)
		.execute(pool)
		.await?;

	Ok(())
}

pub async fn set_deviation(
	pool: &SqlitePool,
	id: &str,
	deviation: f64,
) -> Result<(), sqlx::Error> {
	sqlx::query_scalar!(
		"UPDATE users SET deviation = $1 WHERE id = $2",
		deviation,
		id
	)
	.execute(pool)
	.await?;

	Ok(())
}

pub async fn set_volatility(
	pool: &SqlitePool,
	id: &str,
	volatility: f64,
) -> Result<(), sqlx::Error> {
	sqlx::query_scalar!(
		"UPDATE users SET volatility = $1 WHERE id = $2",
		volatility,
		id
	)
	.execute(pool)
	.await?;

	Ok(())
}

pub async fn finalize_match(
	pool: &SqlitePool,
	winner: &str,
	loser: &str,
) -> Result<(f64, f64), sqlx::Error> {
	let player_1_elo = get_elo(pool, winner).await?;
	let player_1_deviation = get_deviation(pool, winner).await?;
	let player_1_volatility = get_volatility(pool, winner).await?;
	let player_2_elo = get_elo(pool, loser).await?;
	let player_2_deviation = get_deviation(pool, loser).await?;
	let player_2_volatility = get_volatility(pool, loser).await?;

	// DEFAULT_RATING: 1500.0, DEFAULT_DEVIATION: 350.0, DEFAULT_VOLATILITY: 0.06
	// DEFAULT_VOLATILITY_CHANGE: 0.75, DEFAULT_CONVERGENCE_TOLERANCE: 0.000_001

	let parameters =
		Parameters::new(PublicRating::new(1500.0, 250.0, 0.06), 0.75, 0.000_001);

	// DEFAULT_TIME_PERIOD: 60 seconds

	let mut engine = RatingEngine::start_new(Duration::from_secs(60), parameters);

	let r_w = PublicRating::new(player_1_elo, player_1_deviation, player_1_volatility);
	let player_1 = engine.register_player(r_w).0;

	let r_l = PublicRating::new(player_2_elo, player_2_deviation, player_2_volatility);
	let player_2 = engine.register_player(r_l).0;

	engine.register_result(player_1, player_2, &MatchResult::Win);

	let r_w_new: PublicRating = engine.player_rating(player_1).0;
	let r_l_new: PublicRating = engine.player_rating(player_2).0;

	set_elo(pool, winner, r_w_new.rating()).await?;
	set_deviation(pool, winner, r_w_new.deviation()).await?;
	set_volatility(pool, winner, r_w_new.volatility()).await?;
	set_elo(pool, loser, r_l_new.rating()).await?;
	set_deviation(pool, loser, r_l_new.deviation()).await?;
	set_volatility(pool, loser, r_l_new.volatility()).await?;

	Ok((
		r_w_new.rating() - r_w.rating(),
		r_l_new.rating() - r_l.rating(),
	))
}

pub async fn rankings(pool: &SqlitePool) -> Result<Vec<(String, f64)>, sqlx::Error> {
	let res = sqlx::query!(r#"SELECT * FROM users WHERE ABS(elo) > 1 ORDER BY elo DESC"#)
		.fetch_all(pool)
		.await?;

	Ok(res.into_iter().map(|r| (r.id, r.elo)).collect())
}
