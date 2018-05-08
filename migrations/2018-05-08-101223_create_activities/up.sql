create table activities (
    id serial primary key not null,
    name text not null,
    mode text,
    min_fireteam_size integer not null,
    max_fireteam_size integer not null,
    min_light integer,
    min_level integer
);

create index activities_name_idx on activities(name);

create table activityshortcuts (
    id serial primary key not null,
    name text not null unique,
    game text not null,
    link integer not null references activities(id) on delete restrict
);
