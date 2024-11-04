use {
    crate::{Activities, ActivityShortcuts},
    sea_orm_migration::{prelude::*, schema::*},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // create table activities (
        //     id serial primary key not null,
        //     name text not null,
        //     mode text,
        //     min_fireteam_size integer not null,
        //     max_fireteam_size integer not null,
        //     min_light integer,
        //     min_level integer
        // );
        manager
            .create_table(
                Table::create()
                    .table(Activities::Table)
                    .if_not_exists()
                    .col(pk_auto(Post::Id))
                    .col(string(Activities::Name))
                    .col(string_null(Activities::Mode))
                    .col(integer(Activities::MinFireteamSize))
                    .col(integer(Activities::MaxFireteamSize))
                    .col(integer_null(Activities::MinLight))
                    .col(integer_null(Activities::MinLevel))
                    .to_owned(),
            )
            .await?;
        // create index activities_name_idx on activities(name);
        manager
            .create_index(
                Index::create()
                    .name("activities_name_idx")
                    .table(Activities::Table)
                    .col(Activities::Name)
                    .to_owned(),
            )
            .await?;

        // create table activityshortcuts (
        //     id serial primary key not null,
        //     name text not null unique,
        //     game text not null,
        //     link integer not null references activities(id) on delete restrict
        // );
        manager
            .create_table(
                Table::create()
                    .table(ActivityShortcuts::Table)
                    .if_not_exists()
                    .col(pk_auto(ActivityShortcuts::Id))
                    .col(string_uniq(ActivityShortcuts::Name))
                    .col(string(ActivityShortcuts::Game))
                    .col(integer(ActivityShortcuts::Link))
                    .foreign_key(
                        ForeignKey::create()
                            .name("activityshortcuts_link_fkey")
                            .from(ActivityShortcuts::Table, ActivityShortcuts::Link)
                            .to(Activities::Table, Activities::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // drop index activities_name_idx;
        // drop table activityshortcuts;
        // drop table activities;
        manager
            .drop_index(Index::drop().name("activities_name_idx").to_owned())
            .drop_table(Table::drop().table(ActivityShortcuts::Table).to_owned())
            .drop_table(Table::drop().table(Activities::Table).to_owned())
            .await
    }
}
