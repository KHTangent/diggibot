use std::time::Duration;

use chrono::Timelike;
use log::{info, warn};
use poise::serenity_prelude as serenity;
use sqlx::SqlitePool;

mod leeting;
mod models;

use leeting::{
	handle_leet_message, is_leet_message, leeterboard, post_needed_leaderboards, setup_leet,
};

#[derive(Clone)]
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
			commands: vec![setup_leet(), leeterboard()],
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

fn start_ticker_task(ctx: &serenity::Context, data: &Data) {
	info!("Starting leeterboard checks every minute");
	let data = data.clone();
	let ctx = ctx.clone();
	tokio::spawn(async move {
		let time_now = chrono::Utc::now();
		// Always check xx:xx:01 so people know they had enought ime
		let seconds_until_first_check: u64 = 61 - time_now.second() as u64;
		tokio::time::sleep(Duration::from_secs(seconds_until_first_check)).await;
		let mut interval = tokio::time::interval(Duration::from_secs(60));
		loop {
			interval.tick().await;
			post_needed_leaderboards(&ctx, &data).await.ok();
		}
	});
}

async fn event_handler(
	ctx: &serenity::Context,
	event: &serenity::FullEvent,
	_framework: poise::FrameworkContext<'_, Data, Error>,
	data: &Data,
) -> Result<(), Error> {
	match event {
		serenity::FullEvent::CacheReady { .. } => {
			start_ticker_task(ctx, data);
		}
		serenity::FullEvent::Ready { data_about_bot, .. } => {
			info!("Logged in as {}", data_about_bot.user.name);
		}
		serenity::FullEvent::Message { new_message } => {
			if !new_message.author.bot && is_leet_message(&new_message.content) {
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
