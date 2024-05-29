use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use dotenv::dotenv;
use std::{env, str::FromStr};

mod models;
use crate::models::raw_tx::*;
use serde::{Serialize, Deserialize};

mod schema;
use schema::raw_tx::dsl::*;
use diesel::prelude::*;
use diesel::pg::PgConnection;



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

    let connection = &mut establish_connection();
    let results = raw_tx
        .limit(1)
        .select(RawTx::as_select())
        .load(connection)
        .expect("Error loading posts");

    for transaction in results {
        tracing::info!("{:#?}", transaction);
    }


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
            tracing::info!("A");

            let log_message = log.trim_start_matches("Program log: ");

            // Deserialize JSON log messages
            match serde_json::from_str::<EncodedBoard>(log_message) {
                Ok(decoded_board) => {

                    // tracing::info!("{:?}", decoded_board.Board);

                },
                Err(e) => tracing::info!("Failed to deserialize state log: {}, error: {}", log_message, e),
            }
        }

    }

    StatusCode::OK
}



pub fn establish_connection() -> PgConnection {

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}



/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
