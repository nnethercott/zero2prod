-- Add migration script here
insert into users(user_id, name, password_hash)
values(
  'c85b2407-a2f6-40c5-9c2d-7a92c2dd1adc',
  'admin',
 '$argon2id$v=19$m=15000,t=2,p=1$0Dslr9P1AO6W+YeLXHqB1A$ws7piO0DNLEWh3Bjna8505W29UMu1sLrHEktzVHja6U'
);
