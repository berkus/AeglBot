use {
    crate::{Activities, ActivityShortcuts},
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
                    .table(Activities::Table) // create table activities (
                    .col(pk_auto(Activities::Id)) // id serial primary key not null,
                    .col(string(Activities::Name)) // name text not null,
                    .col(string_null(Activities::Mode)) // mode text,
                    .col(integer(Activities::MinFireteamSize)) // min_fireteam_size integer not null,
                    .col(integer(Activities::MaxFireteamSize)) // max_fireteam_size integer not null,
                    .col(integer_null(Activities::MinLight)) // min_light integer,
                    .col(integer_null(Activities::MinLevel)) // min_level integer
                    .to_owned(),
            )
            .await?;
        // create index activities_name_idx on activities(name);
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("activities_name_idx")
                    .table(Activities::Table)
                    .col(Activities::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(ActivityShortcuts::Table) // create table activityshortcuts (
                    .col(pk_auto(ActivityShortcuts::Id)) // id serial primary key not null,
                    .col(string_uniq(ActivityShortcuts::Name)) // name text not null unique,
                    .col(string(ActivityShortcuts::Game)) // game text not null,
                    .col(integer(ActivityShortcuts::Link)) // link integer not null references activities(id) on delete restrict
                    .foreign_key(
                        ForeignKey::create()
                            .name("activityshortcuts_link_fkey")
                            .from(ActivityShortcuts::Table, ActivityShortcuts::Link)
                            .to(Activities::Table, Activities::Id)
                            .on_delete(ForeignKeyAction::Cascade) // Drop all shortcuts if activity is dropped.
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // drop index activities_name_idx;
        manager
            .drop_index(Index::drop().name("activities_name_idx").to_owned())
            .await?;
        // drop table activityshortcuts;
        manager
            .drop_table(Table::drop().table(ActivityShortcuts::Table).to_owned())
            .await?;
        // drop table activities;
        manager
            .drop_table(Table::drop().table(Activities::Table).to_owned())
            .await
    }
}
