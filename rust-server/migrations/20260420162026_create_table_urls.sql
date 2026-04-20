-- Add migration script here
create table urls (
  id bigserial primary key,
  url_id varchar(100) unique,
  full_url varchar(1000) unique,
  date date
  );
