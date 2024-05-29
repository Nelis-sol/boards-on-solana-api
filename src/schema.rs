// @generated automatically by Diesel CLI.

diesel::table! {
    raw_tx (id) {
        id -> Int4,
        tx -> Nullable<Text>,
        ts -> Timestamp,
    }
}
