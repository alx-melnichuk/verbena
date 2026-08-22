-- Adding entities: "user_registration", "user_recovery".

-- **

/* Creating the "user_registration" table. */
CREATE TABLE user_registration (
    id SERIAL PRIMARY KEY NOT NULL,
    nickname VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    "password" VARCHAR(255) NOT NULL,
    final_date TIMESTAMPTZ NOT NULL
);

/* Create indexes for the "user_registration" table. */
CREATE INDEX idx_user_registration_final_date_nickname ON user_registration(final_date, nickname);
CREATE INDEX idx_user_registration_final_date_email ON user_registration(final_date, email);
CREATE INDEX idx_user_registration_final_date ON user_registration(final_date);

-- **

/* Creating the "user_recovery" table. */
CREATE TABLE user_recovery (
    id SERIAL PRIMARY KEY NOT NULL,
    user_id INT REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    final_date TIMESTAMPTZ NOT NULL
);

/* Create indexes for the "user_recovery" table. */
CREATE INDEX idx_user_recovery_user_id_final_date ON user_recovery(user_id, final_date);
CREATE INDEX idx_user_recovery_final_date ON user_recovery(final_date);

-- **
