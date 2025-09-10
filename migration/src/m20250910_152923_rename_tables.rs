use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .rename_table(
                Table::rename()
                    .table(
                        Alias::new("activityshortcuts"),
                        Alias::new("activity_shortcuts"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(
                        Alias::new("plannedactivities"),
                        Alias::new("planned_activities"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(
                        Alias::new("plannedactivitymembers"),
                        Alias::new("planned_activity_members"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .rename_table(
                Table::rename()
                    .table(
                        Alias::new("activity_shortcuts"),
                        Alias::new("activityshortcuts"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(
                        Alias::new("planned_activities"),
                        Alias::new("plannedactivities"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(
                        Alias::new("planned_activity_members"),
                        Alias::new("plannedactivitymembers"),
                    )
                    .to_owned(),
            )
            .await
    }
}
