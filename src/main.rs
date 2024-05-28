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
    tracing::info!("{}", database_url);

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

            tracing::info!("B");

            // Connect to the database.
            let (client, connection) =
                tokio_postgres::connect(&database_url, NoTls)
                    .await
                    .expect("could not connect to databse");

            tracing::info!("C");

            // The connection object performs the actual communication with the database,
            // so spawn it off to run on its own.
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("connection error: {}", e);
                }
            });

            tracing::info!("D");

            // Now we can execute a simple statement that just returns its parameter.
            let rows = client
                .query("SELECT * FROM helius", &[])
                .await
                .unwrap();

            tracing::info!("E");
            
            // And then check that we got back the same string we sent over.
            let value: &str = rows[0].get(0);

            tracing::info!("Query result: {}", value);

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



type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;


/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
