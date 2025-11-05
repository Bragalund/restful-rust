#![no_main]

use libfuzzer_sys::fuzz_target;

// Use your crate as a library. In Cargo.toml, ensure the main crate has a [lib] section or path,
// or use `extern crate` if needed.
use restful_rust::routes;
use restful_rust::schema::{Db, Game, Genre};

use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use warp::http::Method;

fn mocked_db() -> Db {
    Arc::new(Mutex::new(vec![
        Game {
            id: 1,
            title: String::from("Crappy title"),
            rating: 35,
            genre: Genre::RolePlaying,
            description: Some(String::from("Test description...")),
            release_date: chrono::NaiveDate::from_ymd_opt(2011, 9, 22)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        },
        Game {
            id: 2,
            title: String::from("Decent game"),
            rating: 84,
            genre: Genre::Strategy,
            description: None,
            release_date: chrono::NaiveDate::from_ymd_opt(2014, 3, 11)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        },
    ]))
}

fn bytes_to_method_and_path(data: &[u8]) -> (Method, String) {
    // Simple dispatcher from bytes:
    // 0 => GET /games?[offset]&[limit]
    // 1 => POST /games body = data
    // 2 => PUT /games/{id} body = data
    // 3 => DELETE /games/{id}
    if data.is_empty() {
        return (Method::GET, "/games".to_string());
    }
    let selector = data[0] % 4;
    match selector {
        0 => {
            // derive small numbers for offset/limit
            let off = if data.len() > 1 { (data[1] % 5) as u8 } else { 0 };
            let lim = if data.len() > 2 { (data[2] % 5) as u8 } else { 3 };
            (
                Method::GET,
                format!("/games?offset={off}&limit={lim}"),
            )
        }
        1 => (Method::POST, "/games".to_string()),
        2 => {
            let id = if data.len() > 1 { (data[1] % 10) as u8 } else { 0 };
            (Method::PUT, format!("/games/{id}"))
        }
        _ => {
            let id = if data.len() > 1 { (data[1] % 10) as u8 } else { 0 };
            (Method::DELETE, format!("/games/{id}"))
        }
    }
}

fn fuzz_body_json(data: &[u8]) -> Vec<u8> {
    // Try to form somewhat plausible JSON; let fuzzer mutate fields arbitrarily
    // If data isn't UTF-8, just pass raw bytes which will exercise JSON errors.
    if let Ok(s) = std::str::from_utf8(data) {
        // Wrap into a Game-like structure with fuzzed fields
        let payload = format!(
            r#"{{"id":{},"title":"{}","rating":{},"genre":"{}","description":{},"releaseDate":"{}"}}"#,
            3,
            s.chars().take(32).collect::<String>(),
            (s.len() % 101),
            if s.len() % 2 == 0 { "STRATEGY" } else { "ROLE_PLAYING" },
            if s.len() % 3 == 0 {
                "null".to_string()
            } else {
                format!(r#""{}""#, s.chars().rev().take(16).collect::<String>())
            },
            "2016-03-11T00:00:00"
        );
        payload.into_bytes()
    } else {
        data.to_vec()
    }
}

fuzz_target!(|data: &[u8]| {
    // Build a single-threaded tokio runtime per iteration (cheap for fuzzing).
    // For performance, you can move this to a lazy_static if needed.
    let mut rt = Runtime::new().unwrap();

    rt.block_on(async {
        let db = mocked_db();
        let filter = routes::games_routes(db.clone());

        let (method, path) = bytes_to_method_and_path(data);

        // Choose body for methods that accept payload
        let mut req = warp::test::request().method(method.as_str()).path(&path);
        match method {
            Method::POST | Method::PUT => {
                // Randomly decide to send oversized content-length to hit 413 paths
                if !data.is_empty() && data[0] % 5 == 0 {
                    req = req.header("content-length", 1024 * 40);
                } else {
                    let body = fuzz_body_json(&data[1..]);
                    req = req.header("content-type", "application/json").body(body);
                }
            }
            _ => {}
        }

        // Exercise the filter; ignore the result but drive the code paths.
        let _ = req.reply(&filter).await;
    });
});
