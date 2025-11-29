// API handlers, the ends of each filter chain

use log::{debug, error};
use std::convert::Infallible;
use warp::{http::StatusCode, Reply};

use crate::schema::{Db, Game, ListOptions};

// `GET /games`
// Returns JSON array of todos
// Allows pagination, for example: `GET /games?offset=10&limit=5`
pub async fn list_games(options: ListOptions, db: Db) -> Result<impl Reply, Infallible> {
    debug!("list all games");

    let offset = options.offset.unwrap_or(0) as i64;
    let limit = options
        .limit
        .unwrap_or(std::usize::MAX)
        .min(std::i64::MAX as usize) as i64;

    let result = sqlx::query_as::<_, Game>(
        r#"
        SELECT id, title, rating, genre, description, release_date
        FROM games
        ORDER BY id
        LIMIT ? OFFSET ?
    "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&db)
    .await;

    match result {
        Ok(games) => Ok(warp::reply::json(&games).into_response()),
        Err(err) => {
            error!("Failed to fetch games: {}", err);
            Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

// `POST /games`
// Create new game entry with JSON body
pub async fn create_game(new_game: Game, db: Db) -> Result<impl Reply, Infallible> {
    debug!("create new game: {:?}", new_game);

    let result = sqlx::query(
        r#"
        INSERT INTO games (id, title, rating, genre, description, release_date)
        VALUES (?, ?, ?, ?, ?, ?)
    "#,
    )
    .bind(new_game.id)
    .bind(new_game.title)
    .bind(new_game.rating)
    .bind(new_game.genre)
    .bind(new_game.description)
    .bind(new_game.release_date)
    .execute(&db)
    .await;

    match result {
        Ok(_) => Ok(StatusCode::CREATED.into_response()),
        Err(sqlx::Error::Database(err)) => {
            debug!("game of given id already exists or invalid: {}", err);
            Ok(StatusCode::BAD_REQUEST.into_response())
        }
        Err(err) => {
            error!("Failed to insert game: {}", err);
            Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

// `PUT /games/:id`
pub async fn update_game(id: i64, updated_game: Game, db: Db) -> Result<impl Reply, Infallible> {
    debug!("update existing game: id={}, game={:?}", id, updated_game);

    let result = sqlx::query(
        r#"
        UPDATE games
        SET title = ?, rating = ?, genre = ?, description = ?, release_date = ?
        WHERE id = ?
    "#,
    )
    .bind(updated_game.title)
    .bind(updated_game.rating)
    .bind(updated_game.genre)
    .bind(updated_game.description)
    .bind(updated_game.release_date)
    .bind(id)
    .execute(&db)
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                debug!("game of given id not found");
                Ok(StatusCode::NOT_FOUND.into_response())
            } else {
                Ok(StatusCode::OK.into_response())
            }
        }
        Err(sqlx::Error::Database(err)) => {
            debug!("conflict while updating game: {}", err);
            Ok(StatusCode::BAD_REQUEST.into_response())
        }
        Err(err) => {
            error!("Failed to update game: {}", err);
            Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

// `DELETE /games/:id`
pub async fn delete_game(id: i64, db: Db) -> Result<impl Reply, Infallible> {
    debug!("delete game: id={}", id);

    let result = sqlx::query(
        r#"
        DELETE FROM games
        WHERE id = ?
    "#,
    )
    .bind(id)
    .execute(&db)
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                debug!("game of given id not found");
                Ok(StatusCode::NOT_FOUND.into_response())
            } else {
                Ok(StatusCode::NO_CONTENT.into_response())
            }
        }
        Err(err) => {
            error!("Failed to delete game: {}", err);
            Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}
