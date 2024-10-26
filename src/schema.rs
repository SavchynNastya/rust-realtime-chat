// @generated automatically by Diesel CLI.

diesel::table! {
    chats (id) {
        id -> Nullable<Integer>,
        user1_id -> Integer,
        user2_id -> Integer,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    files (id) {
        id -> Nullable<Integer>,
        message_id -> Integer,
        file_path -> Text,
        file_type -> Text,
        file_size -> Integer,
        timestamp -> Nullable<Timestamp>,
    }
}

diesel::table! {
    messages (id) {
        id -> Nullable<Integer>,
        chat_id -> Integer,
        user_id -> Integer,
        content -> Nullable<Text>,
        timestamp -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Nullable<Integer>,
        username -> Text,
        hashed_password -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(files -> messages (message_id));
diesel::joinable!(messages -> chats (chat_id));
diesel::joinable!(messages -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    chats,
    files,
    messages,
    users,
);
