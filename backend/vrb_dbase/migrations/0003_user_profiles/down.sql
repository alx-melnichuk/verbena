-- **

/* Remove stored function to modify a user and their profile. */
DROP FUNCTION IF EXISTS modify_user_profile;

-- **

/* Drop trigger "trg_aft_ins_user_ins_profile". */
DROP TRIGGER IF EXISTS trg_aft_ins_user_ins_profile ON users;

/* Remove function "fn_aft_ins_user_ins_profile". */
DROP FUNCTION IF EXISTS fn_aft_ins_user_ins_profile;

-- **

/* Remove the "profiles" table. */
DROP TABLE IF EXISTS profiles;

-- ** 
