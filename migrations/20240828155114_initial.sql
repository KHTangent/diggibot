PRAGMA foreign_keys = ON;

CREATE TABLE servers (
	guild_id TEXT NOT NULL PRIMARY KEY UNIQUE
);

CREATE TABLE leet_setups (
	guild_id TEXT NOT NULL PRIMARY KEY UNIQUE REFERENCES servers(guild_id) ON DELETE CASCADE,
	timezone TEXT NOT NULL DEFAULT "Europe/Oslo",
	leaderboard_channel TEXT NOT NULL,
	leaderboard_count INT NOT NULL DEFAULT 10,
	accept_emoji TEXT NOT NULL,
	deny_emoji TEXT NOT NULL,
	repeat_emoji TEXT NOT NULL
);

CREATE TABLE leets (
	guild_id TEXT NOT NULL REFERENCES servers(guild_id) ON DELETE CASCADE,
	user_id TEXT NOT NULL,
	timestamp INTEGER NOT NULL
);
