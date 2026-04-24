-- Add migration script here
create index if not exists idx_url_date on urls(date);
create index if not exists idx_url_id on urls(url_id);
create index if not exists idx_full_url on urls(full_url);
