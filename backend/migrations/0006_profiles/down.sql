/* Remove stored function to delete a user, their profile, and their session. */
DROP FUNCTION IF EXISTS delete_profile_user;

/* Remove stored function to modify a user and their profile. */
DROP FUNCTION IF EXISTS modify_profile_user;

/* Remove stored function to add a new user. */
DROP FUNCTION IF EXISTS create_profile_user;

/* Drop stored function for retrieving data from the "profiles", "password" and "users" tables by id or nickname, email. */
DROP FUNCTION IF EXISTS find_profile_user;

/* Remove the "profiles" table. */
DROP TABLE IF EXISTS profiles;

