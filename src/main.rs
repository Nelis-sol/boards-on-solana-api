use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

use dotenv::dotenv;
use std::{env, str::FromStr};

mod models;
use crate::models::raw_tx::*;
use serde::{Serialize, Deserialize};




#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();


    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/health_check", get(health_check))
        .route("/raw_tx", post(update_board));


    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}



// basic handler that responds with a static string
async fn root() -> &'static str {
    "Welcome to the Boards on Solana API!"
}

async fn health_check() -> impl IntoResponse {

    StatusCode::OK
}

async fn update_board(
    Json(payload): Json<RawTransactions>,
) -> impl IntoResponse {
    
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    for transaction in payload.0 {

        let (instruction_logs, state_logs): (Vec<&String>, Vec<&String>) = transaction
            .meta
            .logMessages
            .iter()
            .filter(|msg| msg.starts_with("Program log"))
            .partition(|msg| msg.starts_with("Program log: Instruction:"));
    

        // Process and log the instruction logs
        for log in instruction_logs {
            tracing::info!("{}", log);
        }

        // Process and log the state logs
        for log in state_logs {
            let log_message = log.trim_start_matches("Program log: ");

            // Deserialize JSON log messages
            match serde_json::from_str::<EncodedBoard>(log_message) {
                Ok(decoded_board) => {
                    // tracing::info!("{:?}", decoded_board.Board);

                    let config = tokio_postgres::config::Config::from_str(&database_url).unwrap();

                    // set up connection pool
                    let manager = PostgresConnectionManager::new(config, NoTls);
                    let pool = Pool::builder().build(manager).await.unwrap();
                    let conn = pool.get().await.map_err(internal_error).unwrap();

                    tracing::info!("Start query");
                    let row = conn
                        .query_one("select 1 + 1", &[])
                        .await
                        .map_err(internal_error)
                        .unwrap();
                    let two: i32 = row.try_get(0).map_err(internal_error).unwrap();
                    tracing::info!("result of db query: {}", two);

                },
                Err(e) => tracing::info!("Failed to deserialize state log: {}, error: {}", log_message, e),
            }
        }

    }

    StatusCode::OK
}



type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;


/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
