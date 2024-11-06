use crate::{models::server::Server, Context, Error};
use ::serenity::all::{ArgumentConvert, Emoji};
use chrono_tz::Tz;
use poise::serenity_prelude::{self as serenity};

async fn get_emoji_id_or_name(ctx: Context<'_>, emoji: &str) -> Option<String> {
	if emojis::get(emoji).is_some() {
		return Some(emoji.to_string());
	}
	let emoji_parsed = Emoji::convert(ctx, ctx.guild_id(), Some(ctx.channel_id()), emoji)
		.await
		.ok()?;
	Some(format!("<:{}:{}>", emoji_parsed.name, emoji_parsed.id))
}

#[poise::command(
	slash_command,
	guild_only,
	required_permissions = "MANAGE_MESSAGES",
)]
pub async fn setup_leet(
	ctx: Context<'_>,
	#[description = "Emoji for accepted leet"] accept_emoji: String,
	#[description = "Emoji for invalid leet"] invalid_emoji: String,
	#[description = "Emoji for repeated leet"] repeat_emoji: String,
	#[description = "Time zone to use, e.g. Europe/Oslo"] timezone: String,
	#[description = "Channel to post leeterboard to"] channel: serenity::Channel,
) -> Result<(), Error> {
	let guild_id_string = format!("{}", ctx.guild_id().unwrap().get());
	let server = Server::get_or_create(&ctx.data().db, &guild_id_string).await?;

	let old_setup = server.get_leet_setup(&ctx.data().db).await.map_err(|_| "Error reaching database")?;
	if old_setup.is_some() {
		ctx.say("Leet has already been configured").await?;
		return Ok(());
	}

	let accept_emoji_parsed = get_emoji_id_or_name(ctx, &accept_emoji)
		.await
		.ok_or("Invalid accept emoji given. Must be a default emoji, or from this server")?;
	let invalid_emoji_parsed = get_emoji_id_or_name(ctx, &invalid_emoji)
		.await
		.ok_or("Invalid invalid emoji given. Must be a default emoji, or from this server")?;
	let repeated_emoji_parsed = get_emoji_id_or_name(ctx, &repeat_emoji)
		.await
		.ok_or("Invalid repeated emoji given. Must be a default emoji, or from this server")?;

	let timezone_parsed =
		Tz::from_str_insensitive(&timezone).map_err(|_| "Invalid time zone given")?;

	let channel_id_string = format!("{}", channel.id().get());

	server
		.setup_leet(
			&ctx.data().db,
			&timezone_parsed.to_string(),
			&channel_id_string,
			15,
			&accept_emoji_parsed,
			&invalid_emoji_parsed,
			&repeated_emoji_parsed,
		)
		.await?;

	ctx.say("Leet set up successfully").await?;
	Ok(())
}
