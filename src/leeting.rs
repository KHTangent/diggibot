use std::str::FromStr;

use crate::{models::server::Server, Context, Data, Error};
use ::serenity::all::{ArgumentConvert, CreateAllowedMentions, Emoji, Message, ReactionType};
use chrono::{Datelike, Timelike};
use chrono_tz::Tz;
use log::info;
use once_cell::sync::Lazy;
use poise::serenity_prelude::{self as serenity};
use regex::Regex;

pub fn is_leet_message(message: &str) -> bool {
	let leet_regex = Lazy::new(|| Regex::new(r"(?i)(^|[^a-z])leet($|[^a-z])").unwrap());

	leet_regex.is_match(message)
}

async fn get_emoji_id_or_name(ctx: Context<'_>, emoji: &str) -> Option<String> {
	if emojis::get(emoji).is_some() {
		return Some(emoji.to_string());
	}
	let emoji_parsed = Emoji::convert(ctx, ctx.guild_id(), Some(ctx.channel_id()), emoji)
		.await
		.ok()?;
	Some(format!("<:{}:{}>", emoji_parsed.name, emoji_parsed.id))
}

#[poise::command(slash_command, guild_only, required_permissions = "MANAGE_MESSAGES")]
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

	let old_setup = server
		.get_leet_setup(&ctx.data().db)
		.await
		.map_err(|_| "Error reaching database")?;
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

#[poise::command(slash_command, guild_only)]
pub async fn leeterboard(ctx: Context<'_>) -> Result<(), Error> {
	let guild_id_string = ctx.guild_id().unwrap().to_string();
	let server = Server::get_or_create(&ctx.data().db, &guild_id_string).await?;
	let leet_setup = server.get_leet_setup(&ctx.data().db).await?;
	if leet_setup.is_none() {
		ctx.say("No leeting enabled in this server").await?;
		return Ok(());
	}
	let leet_setup = leet_setup.unwrap();
	let configured_timezone = Tz::from_str(&leet_setup.timezone)
		.map_err(|_| format!("Invalid timezone {}", &leet_setup.timezone))?;
	let message_local_timestamp = ctx.created_at().with_timezone(&configured_timezone);
	let leaderboard = server
		.get_montly_leaderboard(
			&ctx.data().db,
			message_local_timestamp.month().into(),
			message_local_timestamp.year().into(),
		)
		.await?;
	if leaderboard.is_empty() {
		ctx.say("No leets this month").await?;
		return Ok(());
	}
	let mut place: u32 = 0;
	let reply = leaderboard
		.into_iter()
		.map(|e| {
			place += 1;
			format!("- {}: <@{}>, {} leets", place, e.user_id, e.count)
		})
		.take(leet_setup.leaderboard_count as usize)
		.collect::<Vec<String>>()
		.join("\n");
	ctx.send(
		poise::CreateReply::default()
			.content(reply)
			.allowed_mentions(CreateAllowedMentions::default().empty_users()),
	)
	.await?;
	Ok(())
}

pub async fn handle_leet_message(
	ctx: &serenity::Context,
	_event: &serenity::FullEvent,
	data: &Data,
	message: &Message,
) -> Result<(), Error> {
	if message.guild_id.is_none() {
		info!("No guild");
		return Ok(());
	}
	let guild = Server::get_or_create(&data.db, &message.guild_id.unwrap().to_string()).await?;
	let leet_setup = match guild.get_leet_setup(&data.db).await? {
		Some(setup) => setup,
		None => return Ok(()),
	};
	let configured_timezone = Tz::from_str(&leet_setup.timezone)
		.map_err(|_| format!("Invalid timezone {}", &leet_setup.timezone))?;
	let message_local_timestamp = message.timestamp.with_timezone(&configured_timezone);

	match (
		message_local_timestamp.hour(),
		message_local_timestamp.minute(),
	) {
		(13, 37) => {
			let user_id = &message.author.id.to_string();
			let leet = guild
				.get_leet(
					&data.db,
					&user_id,
					message_local_timestamp.day().into(),
					message_local_timestamp.month().into(),
					message_local_timestamp.year().into(),
				)
				.await?;
			match leet {
				Some(_) => {
					message
						.react(
							ctx,
							ReactionType::try_from(leet_setup.repeat_emoji.as_str())
								.map_err(|_| "Failed to get repeat emoji")?,
						)
						.await
						.map_err(|_| "Failed to add reaction")?;
				}
				None => {
					guild
						.add_leet(
							&data.db,
							&user_id,
							message_local_timestamp.day().into(),
							message_local_timestamp.month().into(),
							message_local_timestamp.year().into(),
						)
						.await?;
					message
						.react(
							ctx,
							ReactionType::try_from(leet_setup.accept_emoji.as_str())
								.map_err(|_| "Failed to get accept_emoji emoji")?,
						)
						.await
						.map_err(|_| "Failed to add reaction")?;
				}
			}
		}
		(_, _) => {
			message
				.react(
					ctx,
					ReactionType::try_from(leet_setup.deny_emoji.as_str())
						.map_err(|_| "Failed to get deny emoji")?,
				)
				.await
				.map_err(|_| "Failed to add reaction")?;
		}
	};
	Ok(())
}
