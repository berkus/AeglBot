create table plannedactivityreminders (
    id serial primary key not null,
    planned_activity_id integer not null references plannedactivities(id),
    user_id integer not null references guardians(id),
    remind timestamp without time zone not null
);
