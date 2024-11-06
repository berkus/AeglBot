use {
    crate::{Activities, Guardians, PlannedActivities, PlannedActivityMembers},
    sea_orm_migration::{prelude::*, schema::*},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // create table plannedactivities (
        //     id serial primary key not null,
        //     author_id integer not null references guardians(id),
        //     activity_id integer not null references activities(id) on delete restrict on update cascade,
        //     details text,
        //     start timestamp with time zone not null
        // );
        manager
            .create_table(
                Table::create()
                    .table(PlannedActivities::Table)
                    .col(pk_auto(PlannedActivities::Id))
                    .col(string(PlannedActivities::AuthorId))
                    .col(integer(PlannedActivities::ActivityId))
                    .col(string_null(PlannedActivities::Details))
                    .col(timestamp_with_time_zone(PlannedActivities::Start))
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

        // create table plannedactivitymembers (
        //     id serial primary key not null,
        //     planned_activity_id integer not null references plannedactivities(id),
        //     user_id integer not null references guardians(id),
        //     added timestamp with time zone not null default now(),
        //     unique (planned_activity_id, user_id)
        // );
        manager
            .create_table(
                Table::create()
                    .table(PlannedActivityMembers::Table)
                    .col(pk_auto(PlannedActivityMembers::Id))
                    .col(integer(PlannedActivityMembers::PlannedActivityId))
                    .col(integer(PlannedActivityMembers::UserId))
                    .col(
                        timestamp_with_time_zone(PlannedActivityMembers::Added)
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
                            .unique()
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
            .drop_table(Table::drop().table(PlannedActivities::Table).to_owned())
            .await
    }
}
