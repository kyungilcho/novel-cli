diesel::table! {
    notes (id) {
        id -> BigInt,
        text -> Text,
        done -> Bool,
        priority -> BigInt,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
