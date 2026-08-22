-- Remove entities: "user_registration", "user_recovery".

-- **

/* Remove indexes for the "user_recovery" table. */
DROP INDEX IF EXISTS idx_user_recovery_user_id_final_date;
DROP INDEX IF EXISTS idx_user_recovery_final_date;

/* Remove the "user_recovery" table. */
DROP TABLE IF EXISTS user_recovery;

-- **

/* Remove indexes for the "user_registration" table. */
DROP INDEX IF EXISTS idx_user_registration_final_date_nickname;
DROP INDEX IF EXISTS idx_user_registration_final_date_email;
DROP INDEX IF EXISTS idx_user_registration_final_date;

/* Remove the "user_registration" table. */
DROP TABLE IF EXISTS user_registration;

-- **
