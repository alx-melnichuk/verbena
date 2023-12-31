// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_role"))]
    pub struct UserRole;
}

diesel::table! {
    sessions (user_id) {
        user_id -> Int4,
        num_token -> Nullable<Int4>,
    }
}

diesel::table! {
    user_recovery (id) {
        id -> Int4,
        user_id -> Int4,
        final_date -> Timestamptz,
    }
}

diesel::table! {
    user_registration (id) {
        id -> Int4,
        #[max_length = 255]
        nickname -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        final_date -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UserRole;

    users (id) {
        id -> Int4,
        #[max_length = 255]
        nickname -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        role -> UserRole,
    }
}

diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(user_recovery -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    sessions,
    user_recovery,
    user_registration,
    users,
);
