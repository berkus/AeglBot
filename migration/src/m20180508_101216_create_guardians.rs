use {
    crate::Guardians,
    sea_orm_migration::{prelude::*, schema::*},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(Guardians::Table) // create table guardians (
                    .col(pk_auto(Guardians::Id)) // id serial primary key not null,
                    .col(string_uniq(Guardians::TelegramName)) // telegram_name text not null unique,
                    .col(big_integer_uniq(Guardians::TelegramId)) // telegram_id bigint not null unique,
                    .col(string(Guardians::PsnName)) // psn_name text not null,
                    .col(string_null(Guardians::Email)) // email text,
                    .col(string_null(Guardians::PsnClan)) // psn_clan text,
                    .col(
                        timestamp_with_time_zone(Guardians::CreatedAt) // created_at timestamp with time zone not null default now(),
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Guardians::UpdatedAt) // updated_at timestamp with time zone not null default now(),
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone_null(Guardians::DeletedAt) // deleted_at timestamp with time zone default null,
                            .default(Expr::value("null")),
                    )
                    .col(json_binary_null(Guardians::Tokens)) // tokens jsonb,
                    .col(string_null(Guardians::PendingActivationCode)) // pending_activation_code text
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Guardians::Table).to_owned())
            .await
    }
}
