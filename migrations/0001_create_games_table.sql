CREATE TABLE IF NOT EXISTS games (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    rating INTEGER NOT NULL CHECK(rating BETWEEN 0 AND 100),
    genre TEXT NOT NULL,
    description TEXT,
    release_date TEXT NOT NULL
);
