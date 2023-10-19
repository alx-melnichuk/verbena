/* Creation of the "user_registration" table. */

CREATE TABLE user_registration (
    id SERIAL PRIMARY KEY NOT NULL,
    nickname VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    "password" VARCHAR(255) NOT NULL,
    final_date TIMESTAMP WITH TIME ZONE NOT NULL
);
