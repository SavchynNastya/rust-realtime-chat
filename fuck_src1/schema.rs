// @generated automatically by Diesel CLI.

diesel::table! {
    chat_rooms (id) {
        id -> Nullable<Integer>,
        name -> Text,
        description -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    files (id) {
        id -> Nullable<Integer>,
        filename -> Text,
        filepath -> Text,
        user_id -> Nullable<Integer>,
        room_id -> Nullable<Integer>,
        uploaded_at -> Timestamp,
    }
}

diesel::table! {
    messages (id) {
        id -> Nullable<Integer>,
        content -> Text,
        user_id -> Nullable<Integer>,
        room_id -> Nullable<Integer>,
        sent_at -> Timestamp,
    }
}

diesel::table! {
    user_rooms (id) {
        id -> Nullable<Integer>,
        user_id -> Nullable<Integer>,
        room_id -> Nullable<Integer>,
        joined_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Nullable<Integer>,
        username -> Text,
        email -> Text,
        hashed_password -> Text,
        created_at -> Timestamp,
    }
}

diesel::joinable!(files -> chat_rooms (room_id));
diesel::joinable!(files -> users (user_id));
diesel::joinable!(messages -> chat_rooms (room_id));
diesel::joinable!(messages -> users (user_id));
diesel::joinable!(user_rooms -> chat_rooms (room_id));
diesel::joinable!(user_rooms -> users (user_id));

// diesel::allow_tables_to_appear_in_same_query!(
//     chat_rooms,
//     files,
//     messages,
//     user_rooms,
//     users,
// );
