use sqlx::SqlitePool;

pub async fn user_exists(pool: &SqlitePool, username: &str) -> Result<bool, sqlx::Error> {
	let exists: i64 = sqlx::query_scalar!(
		"SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)",
		username
	)
	.fetch_one(pool)
	.await?;

	Ok(exists == 1)
}

pub async fn create_user(pool: &SqlitePool, username: &str) -> Result<(), sqlx::Error> {
	sqlx::query!(
		r#"
			INSERT INTO users
			VALUES ($1, 0)
		"#,
		username,
	)
	.execute(pool)
	.await?;

	Ok(())
}

pub async fn create_if_user(
	pool: &SqlitePool,
	username: &str,
) -> Result<(), sqlx::Error> {
	if !user_exists(pool, username).await? {
		create_user(pool, username).await?;
	}

	Ok(())
}

pub async fn get_elo(pool: &SqlitePool, username: &str) -> Result<i64, sqlx::Error> {
	create_if_user(pool, username).await?;

	let elo: i64 =
		sqlx::query_scalar!("SELECT elo FROM users WHERE username = $1", username)
			.fetch_one(pool)
			.await?;

	Ok(elo)
}

pub async fn set_elo(
	pool: &SqlitePool,
	username: &str,
	elo: i64,
) -> Result<(), sqlx::Error> {
	sqlx::query_scalar!(
		"UPDATE users SET elo = $1 WHERE username = $2",
		elo,
		username
	)
	.execute(pool)
	.await?;

	Ok(())
}

pub async fn finalize_match(
	pool: &SqlitePool,
	winner: &str,
	loser: &str,
) -> Result<(i64, i64), sqlx::Error> {
	const K: f64 = 5.0;

	let r_w = get_elo(pool, winner).await.unwrap();
	let r_l = get_elo(pool, loser).await.unwrap();

	// Expected score for winner
	let e_w = 1.0 / (1.0 + 10f64.powf((r_l - r_w) as f64 / 400.0));
	let delta = K * (1.0 - e_w);
	let r_w_new = (r_w as f64 + K * delta).floor() as i64;
	let r_l_new = (r_l as f64 - K * delta).floor() as i64;

	set_elo(pool, winner, r_w_new).await.unwrap();
	set_elo(pool, loser, r_l_new).await.unwrap();

	Ok((r_w_new - r_w, r_l_new - r_l))
}

pub async fn rankings(pool: &SqlitePool) -> Result<Vec<(String, i64)>, sqlx::Error> {
	let res = sqlx::query!(r#"SELECT * FROM users WHERE elo != 0 ORDER BY elo DESC"#)
		.fetch_all(pool)
		.await?;

	Ok(res.into_iter().map(|r| (r.username, r.elo)).collect())
}
