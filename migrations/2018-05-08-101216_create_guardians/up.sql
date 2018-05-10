create table guardians (
    id serial primary key not null,
    telegram_name text not null unique,
    telegram_id bigint not null unique,
    psn_name text not null,
    email text,
    psn_clan text,
    created_at timestamp without time zone not null default now(),
    updated_at timestamp without time zone not null default now(),
    deleted_at timestamp without time zone default null,
    tokens jsonb,
    pending_activation_code text
);
