use diesel::prelude::*;

use crate::dbase::db;
use crate::schema::users;
use crate::users::users_models::{self, UserDTO};
use crate::utils::errors::AppError;

/// Run query using Diesel to find user by id and return it.
pub fn find_user_by_id(conn: &mut db::Connection, id: i32) -> Result<Option<UserDTO>, AppError> {
    // Run query using Diesel to find user by id.
    let opt_user_dto: Option<UserDTO> = users::table
        .filter(users::dsl::id.eq(id))
        .first::<users_models::User>(conn)
        .optional()?
        .and_then(|user| Some(UserDTO::from(user)));

    Ok(opt_user_dto)
}

/// Run query using Diesel to find user by nickname and return it.
pub fn find_user_by_nickname(
    conn: &mut db::Connection,
    nickname: &str,
) -> Result<Vec<UserDTO>, AppError> {
    let res_users: Vec<users_models::User> = if nickname.contains("%") {
        users::table
            .filter(users::dsl::nickname.like(nickname))
            .limit(10)
            .select(users_models::User::as_select())
            .load(conn)?
    } else {
        users::table
            .filter(users::dsl::nickname.eq(nickname))
            .limit(10)
            .select(users_models::User::as_select())
            .load(conn)?
    };

    let result: Vec<UserDTO> = res_users.iter().map(|user| UserDTO::from(user.clone())).collect();

    Ok(result)
}

/// Run query using Diesel to add a new user entry.
pub fn create_user(conn: &mut db::Connection, new_user_dto: UserDTO) -> Result<UserDTO, AppError> {
    let mut new_user: UserDTO = new_user_dto.clone();
    UserDTO::clear_optional(&mut new_user);

    let user: users_models::User = diesel::insert_into(users::table)
        .values(new_user)
        .returning(users_models::User::as_returning())
        .get_result(conn)?;

    Ok(UserDTO::from(user))
}

/// Run query using Diesel to full or partially modify the user entry.
pub fn modify_user(
    conn: &mut db::Connection,
    id: i32,
    new_user_dto: UserDTO,
) -> Result<UserDTO, AppError> {
    let mut new_user: UserDTO = new_user_dto.clone();
    UserDTO::clear_optional(&mut new_user);

    let user: users_models::User = diesel::update(users::dsl::users.find(id))
        .set(&new_user)
        .returning(users_models::User::as_returning())
        .get_result(conn)?;

    Ok(UserDTO::from(user))
}

/// Run query using Diesel to delete a user entry.
pub fn delete_user(conn: &mut db::Connection, id: i32) -> Result<usize, AppError> {
    let count: usize = diesel::delete(users::dsl::users.find(id)).execute(conn)?;

    Ok(count)
}
