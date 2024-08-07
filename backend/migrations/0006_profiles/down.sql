/* Remove stored procedure to delete a user, their profile, and their session. */
DROP FUNCTION IF EXISTS delete_profile_user_by_user_id;

/* Remove stored procedure to add a new user. */
DROP FUNCTION IF EXISTS create_profile_user;

/* Drop stored function for retrieving data from the "profiles" and "users" tables by nickname or email. */
DROP FUNCTION IF EXISTS find_profile_user_by_nickname_or_email;

/* Drop stored function to retrieve data from the "profiles" and "users" tables. */
DROP FUNCTION IF EXISTS get_profile_user;

/* Remove the "profiles" table. */
DROP TABLE IF EXISTS profiles;

