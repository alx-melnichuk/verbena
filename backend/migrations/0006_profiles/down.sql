/* Drop stored function to retrieve data from the "profiles" and "users" tables. */
DROP FUNCTION IF EXISTS get_profile_user;

/* Remove the indexes on the "profiles" table. */
DROP INDEX IF EXISTS idx_profiles_user_id;

/* Remove the "profiles" table. */
DROP TABLE IF EXISTS profiles;

