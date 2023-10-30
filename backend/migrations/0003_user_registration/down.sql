/* Deleting the "user_registration" table. */

DROP INDEX IF EXISTS idx_user_registration_nickname_final_date;
DROP INDEX IF EXISTS idx_user_registration_email_final_date;

DROP TABLE IF EXISTS user_registration;

DROP TABLE IF EXISTS user_recovery;