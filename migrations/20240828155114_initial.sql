PRAGMA foreign_keys = ON;

CREATE TABLE servers (
	guild_id INTEGER PRIMARY KEY UNIQUE
);

CREATE TABLE leet_setups (
	guild_id INTEGER PRIMARY KEY UNIQUE REFERENCES servers(guild_id) ON DELETE CASCADE,
	leaderboard_channel INTEGER,
	leaderboard_count INTEGER NOT NULL DEFAULT 10,
	accept_emoji TEXT NOT NULL,
	deny_emoji TEXT NOT NULL,
	repeat_emoji TEXT NOT NULL
);

CREATE TABLE leets (
	guild_id INTEGER,
	user_id INTEGER,
	timestamp INTEGER
);
