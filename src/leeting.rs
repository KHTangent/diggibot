use crate::{Context, Error};
use poise::serenity_prelude::{self as serenity, MessageBuilder};

#[poise::command(slash_command)]
pub async fn setup_leet(
	ctx: Context<'_>,
	#[description = "Emoji for accepted leet"] accept_emoji: serenity::Emoji,
	#[description = "Emoji for invalid leet"] invalid_emoji: serenity::Emoji,
	#[description = "Emoji for repeated leet"] repeat_emoji: serenity::Emoji,
	#[description = "Time zone to use, e.g. Europe/Oslo"] timezone: String,
	#[description = "Channel to post leeterboard to"] channel: serenity::Channel,
) -> Result<(), Error> {
	Ok(())
}
