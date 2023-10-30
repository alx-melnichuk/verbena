/* Creation of the "user_registration" table. */

CREATE TABLE user_registration (
    id SERIAL PRIMARY KEY NOT NULL,
    nickname VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    "password" VARCHAR(255) NOT NULL,
    final_date TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX idx_user_registration_nickname_final_date ON user_registration(nickname, final_date);
CREATE INDEX idx_user_registration_email_final_date ON user_registration(email, final_date);

CREATE TABLE user_recovery (
    id SERIAL PRIMARY KEY NOT NULL,
    user_id INT REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    final_date TIMESTAMP WITH TIME ZONE NOT NULL
);