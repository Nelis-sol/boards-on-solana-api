// @generated automatically by Diesel CLI.

diesel::table! {
    raw_tx (id) {
        id -> Int4,
        ix -> Nullable<Text>,
        tx -> Nullable<Text>,
        ts -> Timestamp,
    }
}
