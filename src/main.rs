use std::env;
use warp::Filter;

use restful_rust::{routes, schema};

#[tokio::main]
async fn main() {
    // Show debug logs by default by setting `RUST_LOG=restful_rust=debug`
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "restful_rust=debug");
    }
    pretty_env_logger::init();

    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://restful.db".to_string());

    let db = schema::init_pool(&database_url)
        .await
        .expect("Failed to initialize database");

    let api = routes::games_routes(db);

    let routes = api.with(warp::log("restful_rust"));

    // Start the server
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
