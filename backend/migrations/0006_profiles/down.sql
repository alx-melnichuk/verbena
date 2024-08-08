/* Remove stored procedure to delete a user, their profile, and their session. */
DROP FUNCTION IF EXISTS delete_profile_user_by_user_id;

/* Remove stored procedure to add a new user. */
DROP FUNCTION IF EXISTS create_profile_user;

/* Drop stored function for retrieving data from the "profiles", "password" and "users" tables by id or nickname, email. */
DROP FUNCTION IF EXISTS find_profile_user_by_id_or_nickname_email;

/* Remove the "profiles" table. */
DROP TABLE IF EXISTS profiles;

