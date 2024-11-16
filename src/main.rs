use log::{info, warn};
use poise::serenity_prelude as serenity;
use sqlx::SqlitePool;

mod leeting;
mod models;

use leeting::{handle_leet_message, is_leet_message, setup_leet};

struct Data {
	db: SqlitePool,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
	dotenv::dotenv().ok();
	env_logger::init();

	let database_url = std::env::var("DATABASE_URL").unwrap_or("sqlite:data.sqlite".to_string());
	info!("Connecting to db {}", database_url);
	let database_pool = SqlitePool::connect(&database_url)
		.await
		.expect("Could not create database connection");
	info!("Connected to DB, running migraions");
	sqlx::migrate!()
		.run(&database_pool)
		.await
		.expect("Failed to run database migrations");
	info!("DB migrated");

	let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
	let intents =
		serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

	let framework = poise::Framework::builder()
		.options(poise::FrameworkOptions {
			commands: vec![setup_leet()],
			event_handler: |ctx, event, framework, data| {
				Box::pin(event_handler(ctx, event, framework, data))
			},
			..Default::default()
		})
		.setup(|ctx, _ready, framework| {
			Box::pin(async move {
				poise::builtins::register_globally(ctx, &framework.options().commands).await?;
				Ok(Data { db: database_pool })
			})
		})
		.build();

	let client = serenity::ClientBuilder::new(token, intents)
		.framework(framework)
		.await;
	info!("Starting bot...");
	client.unwrap().start().await.unwrap();
}

async fn event_handler(
	ctx: &serenity::Context,
	event: &serenity::FullEvent,
	_framework: poise::FrameworkContext<'_, Data, Error>,
	data: &Data,
) -> Result<(), Error> {
	match event {
		serenity::FullEvent::Ready { data_about_bot, .. } => {
			info!("Logged in as {}", data_about_bot.user.name);
		}
		serenity::FullEvent::Message { new_message } => {
			if is_leet_message(&new_message.content) {
				match handle_leet_message(ctx, event, data, &new_message).await {
					Ok(_) => {}
					Err(error) => warn!("Error handling leet message: {}", error),
				}
			}
		}
		_ => {}
	}
	Ok(())
}
