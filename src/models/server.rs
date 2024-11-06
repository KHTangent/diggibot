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
}
