// @generated automatically by Diesel CLI.

diesel::table! {
    head (singleton) {
        singleton -> Nullable<Integer>,
        node_id -> Nullable<Text>,
    }
}

diesel::table! {
    node_parents (node_id, ord) {
        node_id -> Text,
        parent_id -> Text,
        ord -> Integer,
    }
}

diesel::table! {
    nodes (id) {
        id -> Text,
        message -> Text,
        created_at_unix_ms -> BigInt,
    }
}

diesel::joinable!(head -> nodes (node_id));

diesel::allow_tables_to_appear_in_same_query!(head, node_parents, nodes,);
