/* Remove stored procedure to add a new user. */
DROP PROCEDURE IF EXISTS create_user2;

/* Drop stored function to retrieve data from the "profiles" and "users" tables. */
DROP FUNCTION IF EXISTS get_profile_user;

/* Remove the "profiles" table. */
DROP TABLE IF EXISTS profiles;

