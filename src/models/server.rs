use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Server {
	guild_id: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct LeetSetup {
	guild_id: String,
	timezone: String,
	leaderboard_channel: String,
	leaderboard_count: String,
	accept_emoji: String,
	deny_emoji: String,
	repeat_emoji: String,
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
		Ok(sqlx::query_as!(
			Server,
			r#"
			SELECT guild_id
			FROM servers
			WHERE guild_id = ?
			"#,
			guild_id
		)
		.fetch_optional(pool)
		.await?)
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
	) {
	}
}
