use {
    crate::Guardians,
    sea_orm_migration::{prelude::*, schema::*},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Guardians::Table)
                    .add_column(big_integer_null(Guardians::RisingUid))
                    .add_column(string_null(Guardians::RisingNickname))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Guardians::Table)
                    .drop_column(Guardians::RisingUid)
                    .drop_column(Guardians::RisingNickname)
                    .to_owned(),
            )
            .await
    }
}
