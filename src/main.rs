use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum::body::Body;

use dotenv::dotenv;
use std::{env, str::FromStr};

mod models;
use crate::models::raw_tx::*;
use serde::{Serialize, Deserialize};
use serde_json::Value;

mod schema;
use schema::raw_tx::dsl::*;
use diesel::prelude::*;
use diesel::pg::PgConnection;


use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Signature, Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    message::{Message, MessageHeader},
    hash::Hash,
};

use borsh::BorshDeserialize;




#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();


    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/construct_tx", post(construct_tx))
        .route("/post_tx", post(post_tx))
        .route("/health_check", get(health_check));
        // .route("/raw_tx", post(insert_raw_tx))
        // .route("/board/:id", get(get_board));


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

async fn construct_tx(
    Json(payload): Json<Value>,
) -> impl IntoResponse {

    // Define the RPC client
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    let sender = payload.get("sender").expect("could not find sender field in json").as_str().unwrap();
    let receiver = payload.get("receiver").expect("could not find receiver field in json").as_str().unwrap();

    // Define the sender and receiver public keys
    let sender_pubkey = Pubkey::from_str(sender).expect("Failed to parse sender public key");
    let receiver_pubkey = Pubkey::from_str(receiver).expect("Failed to parse receiver public key");

    // Create the instruction to transfer 0.0001 SOL (1 SOL = 1_000_000_000 lamports)
    let lamports_to_transfer = 100_000;
    let transfer_instruction = system_instruction::transfer(&sender_pubkey, &receiver_pubkey, lamports_to_transfer);

    // Create the message
    let message = Message::new(&[transfer_instruction], Some(&sender_pubkey));

    // Create the unsigned transaction
    let recent_blockhash = client.get_latest_blockhash().expect("Failed to get recent blockhash");
    let mut transaction = Transaction::new_unsigned(message);
    transaction.message.recent_blockhash = recent_blockhash;

    let serialised_message = transaction.message.serialize();
    // let serialised_message_base64 = base64::encode(serialised_message);

    Json(serialised_message).into_response()
}


async fn post_tx(
    Json(payload): Json<Value>,
) -> impl IntoResponse {

    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    let message_str = payload.get("message").expect("could not find transaction field in json").as_str().unwrap();
    let external_signature_str = payload.get("signature").expect("could not find signature field in json").as_str().unwrap();

    // // Decode the base64 message string to bytes
    // let message_bytes = base64::decode(message_str).expect("Failed to decode base64 serialized message");

    // // Deserialize the bytes to a Message object using Borsh
    // let message: Message = BorshDeserialize::try_from_slice(&message_bytes).expect("Failed to deserialize message");

    // match deserialize_message(message_str) {
    //     Ok(message) => println!("{:?}", message),
    //     Err(e) => eprintln!("Error deserializing message: {}", e),
    // }


    // let external_signature = Signature::from_str(external_signature_str).expect("Failed to parse external signature");

    // // Attach the external signature to the transaction
    // transaction.signatures.push(external_signature);

    // // Send the signed transaction
    // let signature = client.send_and_confirm_transaction(&transaction).expect("Failed to send transaction");


    // Json(signature).into_response()

    StatusCode::OK
}



// async fn insert_raw_tx(
//     Json(payload): Json<RawTransactions>,
// ) -> impl IntoResponse {

//     for transaction in payload.0 {

//         let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//         let mut connection = PgConnection::establish(&database_url)
//             .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));


//         let instruction_log = transaction
//             .meta
//             .logMessages
//             .iter()
//             .find(|msg| msg.starts_with("Program log: Instruction:"))
//             .unwrap()
//             .to_string();

//         let state_log = transaction
//             .meta
//             .logMessages
//             .iter()
//             .find(|msg| !msg.starts_with("Program log: Instruction:") && msg.starts_with("Program log"))
//             .unwrap()
//             .to_string();


//         let state_log_clean = state_log.trim_start_matches("Program log: ").to_string();

//         let mut board: Board = Board {
//             seed: 12,
//             url: String::from("A"),
//             members: vec![String::from("None")],
//             lists: vec![List{list_id: 1, name: String::from("A"), bounty_payout_percentage: 0}],
//             cards: vec![Card{card_id: 1, list_id: 1, bounty: 0}],
//             currency: String::from("None")
//         };

//         match serde_json::from_str::<EncodedBoard>(&state_log_clean) {
//             Ok(decoded_board) => {
    
//                 board = decoded_board.Board.clone();
    
//                 tracing::info!("{:?}", decoded_board.Board);
    
//             },
//             Err(e) => tracing::info!("Failed to deserialize state log, error: {}", e),
//         }

//         let new_log = NewRawTx {
//             ix: instruction_log,
//             tx: state_log_clean,
//         };

//         diesel::insert_into(raw_tx)
//             .values(&new_log)
//             .execute(&mut connection).unwrap();

//     }

//     StatusCode::OK
// }


// async fn get_board(
//     Path(board_id): Path<i32>,
// ) -> impl IntoResponse {

//     tracing::info!("Request received for board: {:?}", board_id);

//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     let mut connection = PgConnection::establish(&database_url)
//         .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

//     let results = raw_tx
//         .limit(1)
//         .select(RawTx::as_select())
//         .filter(id.eq(board_id))
//         .load(&mut connection)
//         .expect("Error loading posts");

//     let raw_transaction: RawTx = results[0].clone();

//     let mut board: Board = Board {
//         seed: 12,
//         url: String::from("A"),
//         members: vec![String::from("None")],
//         lists: vec![List{list_id: 1, name: String::from("A"), bounty_payout_percentage: 0}],
//         cards: vec![Card{card_id: 1, list_id: 1, bounty: 0}],
//         currency: String::from("None")
//     };


//     match serde_json::from_str::<EncodedBoard>(&raw_transaction.tx.unwrap()) {
//         Ok(decoded_board) => {

//             board = decoded_board.Board.clone();

//             tracing::info!("{:?}", decoded_board.Board);

//         },
//         Err(e) => tracing::info!("Failed to deserialize state log, error: {}", e),
//     }

//     tracing::info!("Response: {:?}", board.clone());

//     Json(board).into_response()


// }


// pub fn establish_connection() -> PgConnection {

//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     PgConnection::establish(&database_url)
//         .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
// }



// /// Utility function for mapping any error into a `500 Internal Server Error`
// /// response.
// fn internal_error<E>(err: E) -> (StatusCode, String)
// where
//     E: std::error::Error,
// {
//     (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
// }


// // For a get endpoint
// //  
// // let results = raw_tx
// //     .limit(1)
// //     .select(RawTx::as_select())
// //     .load(connection)
// //     .expect("Error loading posts");

// // for transaction in results {
// //     tracing::info!("{:#?}", transaction);
// // }

//     // // Process and log the state logs
//     // for log in state_logs {
//     //     tracing::info!("A");

//     //     let log_message = log.trim_start_matches("Program log: ");

//     //     // Deserialize JSON log messages
//     //     match serde_json::from_str::<EncodedBoard>(log_message) {
//     //         Ok(decoded_board) => {

//     //             // tracing::info!("{:?}", decoded_board.Board);

//     //         },
//     //         Err(e) => tracing::info!("Failed to deserialize state log: {}, error: {}", log_message, e),
//     //     }
//     // }



fn deserialize_message(payload: &str) -> Result<Message, serde_json::Error> {
    let message: Message = serde_json::from_str(payload)?;
    Ok(message)
}



