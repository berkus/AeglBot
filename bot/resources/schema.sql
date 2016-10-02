BEGIN TRANSACTION;

CREATE TABLE users (
    id serial primary key,
    telegram_name text not null unique,
    telegram_id int not null unique,
    psn_name text null unique,
    email text null,
    psn_clan text null,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now(),
    deleted_at timestamp null,
    tokens jsonb,
    pending_activation_code text null
);

CREATE TABLE activity (
    id serial primary key,
    name text not null,
    mode text null,
    min_fireteam_size smallint not null,
    max_fireteam_size smallint not null,
    min_light int null,
    min_level int null
);

INSERT INTO activity (name, mode, min_fireteam_size, max_fireteam_size, min_light, min_level)
VALUES
('Vault of Glass', 'normal', 1, 6, 200, 25),
('Vault of Glass', 'hard', 1, 6, 280, 30),
(E'Crota\'s End', 'normal', 1, 6, 200, 30),
(E'Crota\'s End', 'hard', 1, 6, 280, 33),
(E'King\'s Fall', 'normal', 1, 6, 290, 35),
(E'King\'s Fall', 'hard', 1, 6, 320, 40),
('Wrath of the Machine', 'normal', 1, 6, 370, 40),
('Vanguard', 'Patrols', 1, 3, null, null),
('Vanguard', 'any', 1, 3, null, null),
('Crucible', 'Private Matches', 1, 12, null, null),
('Crucible', 'Trials of Osiris', 3, 3, 370, 40),
('Crucible', 'Iron Banner', 1, 6, 370, 40),
('Crucible', '6v6', 1, 6, null, null),
('Crucible', '3v3', 1, 3, null, null),
('Crucible', 'any', 1, 12, null, null);

CREATE TABLE planned_activity (
    id serial primary key,
    author_id int not null references users(id) on delete cascade,
    activity_id int not null references activity(id) on delete cascade,
    details text null,
    start timestamp
);

CREATE TABLE planned_activity_members (
    id serial primary key,
    planned_activity_id int not null references planned_activity(id) on delete cascade,
    user_id int not null references users(id) on delete cascade,
    added timestamp,
    unique (planned_activity_id, user_id)
);

CREATE TABLE planned_activity_reminder (
    id serial primary key,
    planned_activity_id int not null references planned_activity(id) on delete cascade,
    user_id int not null references users(id) on delete cascade,
    remind timestamp
);

COMMIT;
