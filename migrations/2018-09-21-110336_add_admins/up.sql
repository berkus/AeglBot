alter table guardians add column is_admin boolean not null default false;

update guardians set is_admin = true where telegram_name = 'berkus';
