use poise::serenity_prelude as serenity;
use sqlx::SqlitePool;

struct Data {
	db: SqlitePool,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command)]
async fn age(
	ctx: Context<'_>,
	#[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
	let u = user.as_ref().unwrap_or_else(|| ctx.author());
	let response = format!("{}'s account was created at {}", u.name, u.created_at());
	ctx.say(response).await?;
	Ok(())
}

#[tokio::main]
async fn main() {
	dotenv::dotenv().ok();

	let database_url = std::env::var("DATABASE_URL").unwrap_or("sqlite:data.sqlite".to_string());
	let database_pool = SqlitePool::connect(&database_url)
		.await
		.expect("Could not create database connection");
	sqlx::migrate!()
		.run(&database_pool)
		.await
		.expect("Failed to run database migrations");

	let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
	let intents = serenity::GatewayIntents::non_privileged();

	let framework = poise::Framework::builder()
		.options(poise::FrameworkOptions {
			commands: vec![age()],
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
	client.unwrap().start().await.unwrap();
}
