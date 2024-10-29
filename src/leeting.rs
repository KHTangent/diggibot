use crate::{Context, Error};
use ::serenity::all::{ArgumentConvert, Emoji};
use poise::serenity_prelude::{self as serenity};

async fn get_emoji_id_or_name(ctx: Context<'_>, emoji: &str) -> Option<String> {
	println!("Attempting to parse: {}", emoji);
	if emojis::get(emoji).is_some() {
		return Some(emoji.to_string());
	}
	let emoji_parsed = Emoji::convert(ctx, ctx.guild_id(), Some(ctx.channel_id()), emoji)
		.await
		.ok()?;
	Some(format!("<:{}:{}>", emoji_parsed.name, emoji_parsed.id))
}

#[poise::command(slash_command)]
pub async fn setup_leet(
	ctx: Context<'_>,
	#[description = "Emoji for accepted leet"] accept_emoji: String,
	#[description = "Emoji for invalid leet"] invalid_emoji: String,
	#[description = "Emoji for repeated leet"] repeat_emoji: String,
	#[description = "Time zone to use, e.g. Europe/Oslo"] timezone: String,
	#[description = "Channel to post leeterboard to"] channel: serenity::Channel,
) -> Result<(), Error> {
	let accept_emoji_parsed = get_emoji_id_or_name(ctx, &accept_emoji)
		.await
		.ok_or("Invalid accept emoji given. Must be a default emoji, or from this server")?;
	let invalid_emoji_parsed = get_emoji_id_or_name(ctx, &invalid_emoji)
		.await
		.ok_or("Invalid invalid emoji given. Must be a default emoji, or from this server")?;
	let repeated_emoji_parsed = get_emoji_id_or_name(ctx, &repeat_emoji)
		.await
		.ok_or("Invalid repeated emoji given. Must be a default emoji, or from this server")?;
	ctx.say(format!(
		"Using emojis {}, {}, and {}",
		accept_emoji_parsed, invalid_emoji_parsed, repeated_emoji_parsed
	))
	.await?;
	Ok(())
}
