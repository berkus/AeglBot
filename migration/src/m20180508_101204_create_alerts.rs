use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // create table alerts (
        //     id serial primary key not null,
        //     guid text not null unique,
        //     title text not null,
        //     type text not null,
        //     startdate timestamp with time zone not null,
        //     expirydate timestamp with time zone,
        //     faction text,
        //     flavor text
        // );
        manager
            .create_table(
                Table::create()
                    .table(Alerts::Table)
                    .if_not_exists()
                    .col(pk_auto(Alerts::Id))
                    .col(string(Alerts::Guid))
                    .col(string(Alerts::Title))
                    .col(string(Alerts::Title))
                    .col(string(Alerts::Type))
                    .col(timestamp_with_time_zone(Alerts::StartDate))
                    .col(timestamp_with_time_zone_null(Alerts::ExpiryDate))
                    .col(string_null(Alerts::Faction))
                    .col(string_null(Alerts::Flavor))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alerts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Alerts {
    Table,
    Id,
    Guid,
    Title,
    Type,
    StartDate,
    ExpiryDate,
    Faction,
    Flavor,
}
