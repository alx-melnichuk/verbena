/* Creation of the "users" table. */

CREATE TABLE users (
    id SERIAL PRIMARY KEY NOT NULL,
    nickname VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    "password" VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT diesel_manage_updated_at('users');

/*
CREATE TABLE public.users (
	id serial4 NOT NULL,
	nickname varchar(255) NOT NULL,
	email varchar(255) NOT NULL,
	"password" varchar(255) NOT NULL,
	created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
	CONSTRAINT users_email_key UNIQUE (email),
	CONSTRAINT users_nickname_key UNIQUE (nickname),
	CONSTRAINT users_pkey PRIMARY KEY (id)
);*/