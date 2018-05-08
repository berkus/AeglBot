create table alerts (
    id serial primary key not null,
    guid text not null unique,
    title text not null,
    type text not null,
    startdate timestamp without time zone not null,
    expirydate timestamp without time zone,
    faction text
);
