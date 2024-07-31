/* Drop stored function to retrieve data from the "profiles" and "users" tables. */
DROP FUNCTION IF EXISTS get_profile_user;

/* Remove the "profiles" table. */
DROP TABLE IF EXISTS profiles;

