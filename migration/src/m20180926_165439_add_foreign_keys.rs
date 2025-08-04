use {
    crate::{Activities, ActivityShortcuts, Guardians, PlannedActivities, PlannedActivityMembers},
    sea_orm_migration::{prelude::*, schema::*},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        todo!();

        // -- Links between activities and activityshortcuts
        // -- Drop all shortcuts if activity is dropped.
        // alter table activityshortcuts drop constraint if exists activityshortcuts_link_fkey;
        // alter table activityshortcuts add constraint activityshortcuts_link_fkey
        // foreign key (link) references activities(id) on delete cascade on update cascade;

        // -- Links between activities and plannedactivities
        // -- Disallow activity drop if plannedactivities exist.
        // alter table plannedactivities drop constraint if exists plannedactivities_activity_id_fkey;
        // alter table plannedactivities add constraint plannedactivities_activity_id_fkey
        // foreign key (activity_id) references activities(id) on delete restrict on update cascade;

        // -- Links between plannedactivitymembers and plannedactivities
        // -- Drop all members if planned activity is dropped.
        // alter table plannedactivitymembers drop constraint if exists plannedactivitymembers_planned_activity_id_fkey;
        // alter table plannedactivitymembers add constraint plannedactivitymembers_planned_activity_id_fkey
        // foreign key (planned_activity_id) references plannedactivities(id) on delete cascade on update cascade;

        // -- Drop member if guardian is dropped.
        // alter table plannedactivitymembers drop constraint if exists plannedactivitymembers_user_id_fkey;
        // alter table plannedactivitymembers add constraint plannedactivitymembers_user_id_fkey
        // foreign key (user_id) references guardians(id) on delete cascade on update cascade;

        manager
            .create_table(
                Table::create()
                    .table(Post::Table)
                    .if_not_exists()
                    .col(pk_auto(Post::Id))
                    .col(string(Post::Title))
                    .col(string(Post::Text))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        todo!();

        // alter table activityshortcuts drop constraint if exists activityshortcuts_link_fkey;
        // alter table activityshortcuts add constraint activityshortcuts_link_fkey
        // foreign key (link) references activities(id) on delete restrict;

        // alter table plannedactivities drop constraint if exists plannedactivities_activity_id_fkey;
        // alter table plannedactivities add constraint plannedactivities_activity_id_fkey
        // foreign key (activity_id) references activities(id);

        // alter table plannedactivitymembers drop constraint if exists plannedactivitymembers_planned_activity_id_fkey;
        // alter table plannedactivitymembers add constraint plannedactivitymembers_planned_activity_id_fkey
        // foreign key (planned_activity_id) references plannedactivities(id);

        // alter table plannedactivitymembers drop constraint if exists plannedactivitymembers_user_id_fkey;
        // alter table plannedactivitymembers add constraint plannedactivitymembers_user_id_fkey
        // foreign key (user_id) references guardians(id);

        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await
    }
}
