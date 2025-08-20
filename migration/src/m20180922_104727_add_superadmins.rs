use {
    crate::Guardians,
    sea_orm_migration::{prelude::*, schema::*},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // alter table guardians add column is_superadmin boolean not null default false;
        manager
            .alter_table(
                Table::alter()
                    .table(Guardians::Table)
                    .add_column(boolean(Guardians::IsSuperadmin).default(Expr::value(false)))
                    .to_owned(),
            )
            .await?;
        // update guardians set is_superadmin = true where telegram_name = 'berkus';
        let update = Query::update()
            .table(Guardians::Table)
            .value(Guardians::IsSuperadmin, true)
            .and_where(Expr::col(Guardians::TelegramName).eq("berkus"))
            .to_owned();

        manager.exec_stmt(update).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // alter table guardians drop column is_superadmin;
        manager
            .alter_table(
                Table::alter()
                    .table(Guardians::Table)
                    .drop_column(Guardians::IsSuperadmin)
                    .to_owned(),
            )
            .await
    }
}
