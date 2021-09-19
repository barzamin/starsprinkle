-- Add migration script here
create table position_messages (
    id bigserial primary key not null,
    ts timestamp not null,

    messagedata jsonb not null,
    latitude real,
    longitude real
);