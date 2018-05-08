create table plannedactivities (
    id serial primary key not null,
    author_id integer not null references guardians(id),
    activity_id integer not null references activities(id),
    details text,
    start timestamp without time zone not null
);

create table plannedactivitymembers (
    id serial primary key not null,
    planned_activity_id integer not null references plannedactivities(id),
    user_id integer not null references guardians(id),
    added timestamp without time zone not null default now(),
    unique (planned_activity_id, user_id)
);
