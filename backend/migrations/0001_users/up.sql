/* Creation of the "users" table. */

CREATE TABLE users (
    id SERIAL PRIMARY KEY NOT NULL,
    nickname VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    "password" VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT diesel_manage_updated_at('users');

CREATE UNIQUE INDEX uq_idx_users_nickname ON users(nickname);
CREATE UNIQUE INDEX uq_idx_users_email ON users(email);
