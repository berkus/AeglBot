use {
    crate::Alerts,
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
                    .table(Alerts::Table) // create table if not exists alerts (
                    .col(pk_auto(Alerts::Id)) //     id serial primary key not null,
                    .col(string(Alerts::Guid)) //     guid text not null unique,
                    .col(string(Alerts::Title)) //     title text not null,
                    .col(string(Alerts::Type)) //     type text not null,
                    .col(timestamp_with_time_zone(Alerts::StartDate)) //     startdate timestamp with time zone not null,
                    .col(timestamp_with_time_zone_null(Alerts::ExpiryDate)) //     expirydate timestamp with time zone,
                    .col(string_null(Alerts::Faction)) //     faction text,
                    .col(string_null(Alerts::Flavor)) //     flavor text
                    .to_owned(), // );
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alerts::Table).to_owned())
            .await
    }
}
