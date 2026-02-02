-- Song database schema (beatoraja compatible subset)
-- This schema stores metadata for BMS songs.

CREATE TABLE IF NOT EXISTS song (
    sha256 TEXT PRIMARY KEY NOT NULL,
    md5 TEXT NOT NULL DEFAULT '',
    path TEXT NOT NULL DEFAULT '',
    folder TEXT NOT NULL DEFAULT '',
    title TEXT NOT NULL DEFAULT '',
    subtitle TEXT NOT NULL DEFAULT '',
    artist TEXT NOT NULL DEFAULT '',
    subartist TEXT NOT NULL DEFAULT '',
    genre TEXT NOT NULL DEFAULT '',
    mode INTEGER NOT NULL DEFAULT 7,
    level INTEGER NOT NULL DEFAULT 0,
    difficulty INTEGER NOT NULL DEFAULT 0,
    max_bpm INTEGER NOT NULL DEFAULT 0,
    min_bpm INTEGER NOT NULL DEFAULT 0,
    notes INTEGER NOT NULL DEFAULT 0,
    date INTEGER NOT NULL DEFAULT 0,
    add_date INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_song_path ON song(path);
CREATE INDEX IF NOT EXISTS idx_song_md5 ON song(md5);
CREATE INDEX IF NOT EXISTS idx_song_folder ON song(folder);

-- Score database schema (beatoraja compatible subset)
-- This schema stores player scores.

CREATE TABLE IF NOT EXISTS score (
    sha256 TEXT NOT NULL,
    mode INTEGER NOT NULL DEFAULT 0,
    clear INTEGER NOT NULL DEFAULT 0,
    ex_score INTEGER NOT NULL DEFAULT 0,
    max_combo INTEGER NOT NULL DEFAULT 0,
    min_bp INTEGER NOT NULL DEFAULT 2147483647,
    pg INTEGER NOT NULL DEFAULT 0,
    gr INTEGER NOT NULL DEFAULT 0,
    gd INTEGER NOT NULL DEFAULT 0,
    bd INTEGER NOT NULL DEFAULT 0,
    pr INTEGER NOT NULL DEFAULT 0,
    ms INTEGER NOT NULL DEFAULT 0,
    notes INTEGER NOT NULL DEFAULT 0,
    play_count INTEGER NOT NULL DEFAULT 0,
    clear_count INTEGER NOT NULL DEFAULT 0,
    date INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (sha256, mode)
);

CREATE INDEX IF NOT EXISTS idx_score_sha256 ON score(sha256);
