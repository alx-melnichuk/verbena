/* Deleting the "user_registration" table. */

/* user_recovery */

DROP INDEX IF EXISTS idx_user_recovery_user_id_final_date;

DROP TABLE IF EXISTS user_recovery;

/* user_registration */

DROP INDEX IF EXISTS idx_user_registration_final_date_nickname;
DROP INDEX IF EXISTS idx_user_registration_final_date_email;

DROP TABLE IF EXISTS user_registration;
