/* Creation of the "user_registration" table. */

/* user_registration */

CREATE TABLE user_registration (
    id SERIAL PRIMARY KEY NOT NULL,
    nickname VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    "password" VARCHAR(255) NOT NULL,
    final_date TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_user_registration_final_date_nickname ON user_registration(final_date, nickname);
CREATE INDEX idx_user_registration_final_date_email ON user_registration(final_date, email);
CREATE INDEX idx_user_registration_final_date ON user_registration(final_date);


/* user_recovery */

CREATE TABLE user_recovery (
    id SERIAL PRIMARY KEY NOT NULL,
    user_id INT REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    final_date TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX idx_user_recovery_user_id_final_date ON user_recovery(user_id, final_date);
CREATE INDEX idx_user_recovery_final_date ON user_recovery(final_date);
