use {
    crate::Guardians,
    sea_orm_migration::{prelude::*, schema::*},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // alter table guardians add column is_admin boolean not null default false;
        manager
            .alter_table(
                Table::alter()
                    .table(Guardians::Table)
                    .add_column(boolean(Guardians::IsAdmin).default(Expr::value(false)))
                    .to_owned(),
            )
            .await?;
        // update guardians set is_admin = true where telegram_name = 'berkus';
        // entity::Guardians::update_one();
        let update = Query::update()
            .table(Guardians::Table)
            .value(Guardians::IsAdmin, true)
            .and_where(Expr::col(Guardians::TelegramName).eq("berkus"))
            .to_owned();
        manager.exec_stmt(update).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // alter table guardians drop column is_admin;
        manager
            .alter_table(
                Table::alter()
                    .table(Guardians::Table)
                    .drop_column(Guardians::IsAdmin)
                    .to_owned(),
            )
            .await
    }
}
