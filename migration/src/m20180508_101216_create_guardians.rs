use {
    crate::Guardians,
    sea_orm_migration::{prelude::*, schema::*},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // create table guardians (
        //     id serial primary key not null,
        //     telegram_name text not null unique,
        //     telegram_id bigint not null unique,
        //     psn_name text not null,
        //     email text,
        //     psn_clan text,
        //     created_at timestamp with time zone not null default now(),
        //     updated_at timestamp with time zone not null default now(),
        //     deleted_at timestamp with time zone default null,
        //     tokens jsonb,
        //     pending_activation_code text
        // );
        manager
            .create_table(
                Table::create()
                    .table(Guardians::Table)
                    .if_not_exists()
                    .col(pk_auto(Guardians::Id))
                    .col(string_uniq(Guardians::TelegramName))
                    .col(big_integer_uniq(Guardians::TelegramId))
                    .col(string(Guardians::PsnName))
                    .col(string_null(Guardians::Email))
                    .col(string_null(Guardians::PsnClan))
                    .col(
                        timestamp_with_time_zone(Guardians::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Guardians::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone_null(Guardians::DeletedAt)
                            .default(Expr::value("null")),
                    )
                    .col(json_binary_null(Guardians::Tokens))
                    .col(string_null(Guardians::PendingActivationCode))
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
