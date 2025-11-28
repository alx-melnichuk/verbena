-- **

/* Remove stored function to modify a user and their profile. */
DROP FUNCTION IF EXISTS modify_user_profile;

-- **

/* Remove stored function for retrieving data from the "users" and "profiles" tables by ID. */
DROP FUNCTION IF EXISTS get_user_profile_by_id;

-- **

/* Drop trigger "trg_aft_ins_user_ins_profile". */
DROP TRIGGER IF EXISTS trg_aft_ins_user_ins_profile ON users;

/* Remove function "fn_aft_ins_user_ins_profile". */
DROP FUNCTION IF EXISTS fn_aft_ins_user_ins_profile;

-- **

/* Remove the "profiles" table. */
DROP TABLE IF EXISTS profiles;

-- ** 
