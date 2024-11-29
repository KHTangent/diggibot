use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Server {
	guild_id: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct LeetSetup {
	pub guild_id: String,
	pub timezone: String,
	pub leaderboard_channel: String,
	pub leaderboard_count: i64,
	pub accept_emoji: String,
	pub deny_emoji: String,
	pub repeat_emoji: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Leet {
	pub guild_id: String,
	pub user_id: String,
	pub day: i64,
	pub month: i64,
	pub year: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct LeaderboardEntry {
	pub user_id: String,
	pub count: i64,
}

impl Server {
	pub async fn create(pool: &SqlitePool, guild_id: &String) -> Result<Self> {
		let created = sqlx::query_as!(
			Server,
			r#"
			INSERT INTO servers(guild_id)
			VALUES (?)
			RETURNING guild_id
			"#,
			guild_id
		)
		.fetch_one(pool)
		.await?;
		Ok(created)
	}

	pub async fn get(pool: &SqlitePool, guild_id: &String) -> Result<Option<Self>> {
		let found_server_or_none = sqlx::query_as!(
			Server,
			r#"
			SELECT guild_id
			FROM servers
			WHERE guild_id = ?
			"#,
			guild_id
		)
		.fetch_optional(pool)
		.await?;
		Ok(found_server_or_none)
	}

	pub async fn get_or_create(pool: &SqlitePool, guild_id: &String) -> Result<Self> {
		let server = Server::get(pool, guild_id).await?;
		if server.is_none() {
			return Server::create(pool, guild_id).await;
		}
		Ok(server.unwrap())
	}

	pub async fn get_leet_setup(&self, pool: &SqlitePool) -> Result<Option<LeetSetup>> {
		let found_or_none = sqlx::query_as!(
			LeetSetup,
			r#"
			SELECT *
			FROM leet_setups
			WHERE guild_id = ?
			"#,
			self.guild_id
		)
		.fetch_optional(pool)
		.await?;
		Ok(found_or_none)
	}

	pub async fn setup_leet(
		&self,
		pool: &SqlitePool,
		timezone: &String,
		leaderboard_channel: &String,
		leaderboard_count: i32,
		accept_emoji: &String,
		deny_emoji: &String,
		repeat_emoji: &String,
	) -> Result<()> {
		sqlx::query!(
			r#"
			INSERT INTO leet_setups(
				guild_id,
				timezone,
				leaderboard_channel,
				leaderboard_count,
				accept_emoji,
				deny_emoji,
				repeat_emoji
			) VALUES (?, ?, ?, ?, ?, ?, ?)
			"#,
			self.guild_id,
			timezone,
			leaderboard_channel,
			leaderboard_count,
			accept_emoji,
			deny_emoji,
			repeat_emoji
		)
		.execute(pool)
		.await?;
		Ok(())
	}

	pub async fn get_leet(
		&self,
		pool: &SqlitePool,
		user_id: &str,
		day: i64,
		month: i64,
		year: i64,
	) -> Result<Option<Leet>> {
		let leet = sqlx::query_as!(
			Leet,
			r#"
			SELECT *
			FROM leets
			WHERE guild_id = ? AND user_id = ? AND day = ? AND month = ? AND year = ?
			"#,
			self.guild_id,
			user_id,
			day,
			month,
			year
		)
		.fetch_optional(pool)
		.await?;
		Ok(leet)
	}

	pub async fn add_leet(
		&self,
		pool: &SqlitePool,
		user_id: &str,
		day: i64,
		month: i64,
		year: i64,
	) -> Result<()> {
		sqlx::query!(
			r#"
			INSERT INTO leets(guild_id, user_id, day, month, year)
			VALUES (?, ?, ?, ?, ?)
			"#,
			self.guild_id,
			user_id,
			day,
			month,
			year
		)
		.execute(pool)
		.await?;
		Ok(())
	}

	pub async fn get_montly_leaderboard(
		&self,
		pool: &SqlitePool,
		month: i64,
		year: i64,
	) -> Result<Vec<LeaderboardEntry>> {
		let entries: Vec<LeaderboardEntry> = sqlx::query_as!(
			LeaderboardEntry,
			r#"
			SELECT user_id, COUNT(*) as count
			FROM leets
			WHERE guild_id = ? AND month = ? AND year = ?
			GROUP BY user_id
			ORDER BY count DESC
			"#,
			self.guild_id,
			month,
			year
		)
		.fetch_all(pool)
		.await?;
		Ok(entries)
	}
}
