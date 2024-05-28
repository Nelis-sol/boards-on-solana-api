use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Meta {
    pub logMessages: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub accountKeys: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub message: Message,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawTransaction {
    pub meta: Meta,
    pub transaction: Transaction,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawTransactions(pub Vec<RawTransaction>);


#[derive(Debug, Deserialize)]
pub struct EncodedBoard {
    pub Board: Board,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Board {
    pub seed: u32,
    pub url: String,
    pub members: Vec<String>,
    pub lists: Vec<List>,
    pub cards: Vec<Card>,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct List {
    pub list_id: u32,
    pub name: String,
    pub bounty_payout_percentage: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    pub card_id: u32,
    pub list_id: u32,
    pub bounty: u64,
}
