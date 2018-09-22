alter table guardians add column is_superadmin boolean not null default false;

update guardians set is_superadmin = true where telegram_name = 'berkus';
