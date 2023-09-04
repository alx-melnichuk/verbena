/* Creation of the "users" table. */

CREATE TABLE users (
    id SERIAL PRIMARY KEY NOT NULL,
    nickname VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT diesel_manage_updated_at('users');

-- CREATE UNIQUE INDEX users_username_key ON public.users USING btree (username)
-- CREATE UNIQUE INDEX idx_employees_mobile_phone ON employees(mobile_phone);
-- CREATE UNIQUE INDEX username_unique_idx ON users (username)

