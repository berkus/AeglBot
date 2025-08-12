use {
    crate::{Activities, Guardians, PlannedActivities, PlannedActivityMembers},
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
                    .table(PlannedActivities::Table) // create table plannedactivities (
                    .col(pk_auto(PlannedActivities::Id)) // id serial primary key not null,
                    .col(string(PlannedActivities::AuthorId)) // author_id integer not null references guardians(id),
                    .col(integer(PlannedActivities::ActivityId)) // activity_id integer not null references activities(id) on delete restrict on update cascade,
                    .col(string_null(PlannedActivities::Details)) // details text,
                    .col(timestamp_with_time_zone(PlannedActivities::Start)) // start timestamp with time zone not null
                    .foreign_key(
                        ForeignKey::create()
                            .name("plannedactivities_author_id_fkey")
                            .from(PlannedActivities::Table, PlannedActivities::AuthorId)
                            .to(Guardians::Table, Guardians::Id),
                    )
                    // -- Disallow activity drop if plannedactivities exist.
                    .foreign_key(
                        ForeignKey::create()
                            .name("plannedactivities_activity_id_fkey")
                            .from(PlannedActivities::Table, PlannedActivities::ActivityId)
                            .to(Activities::Table, Activities::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(PlannedActivityMembers::Table) // create table plannedactivitymembers (
                    .col(pk_auto(PlannedActivityMembers::Id)) //     id serial primary key not null,
                    .col(integer(PlannedActivityMembers::PlannedActivityId)) // planned_activity_id integer not null references plannedactivities(id),
                    .col(integer(PlannedActivityMembers::UserId)) // user_id integer not null references guardians(id),
                    .col(
                        timestamp_with_time_zone(PlannedActivityMembers::Added) // added timestamp with time zone not null default now(),
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("plannedactivitymembers_planned_activity_id_fkey")
                            .from(
                                PlannedActivityMembers::Table,
                                PlannedActivityMembers::PlannedActivityId,
                            )
                            .to(PlannedActivities::Table, PlannedActivities::Id)
                            .on_delete(ForeignKeyAction::Cascade) // Drop all members if planned activity is dropped.
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("plannedactivitymembers_user_id_fkey")
                            .from(
                                PlannedActivityMembers::Table,
                                PlannedActivityMembers::UserId,
                            )
                            .to(Guardians::Table, Guardians::Id)
                            .on_delete(ForeignKeyAction::Cascade) // Drop member if guardian is dropped.
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .unique() // unique (planned_activity_id, user_id)
                            .name("plannedactivitymembers_planned_activity_id_user_id_key")
                            .col(PlannedActivityMembers::PlannedActivityId)
                            .col(PlannedActivityMembers::UserId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // drop table plannedactivitymembers;
        // drop table plannedactivities;
        manager
            .drop_table(
                Table::drop()
                    .table(PlannedActivityMembers::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(PlannedActivities::Table).to_owned())
            .await
    }
}
