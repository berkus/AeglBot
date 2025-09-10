pub use sea_orm_migration::prelude::*;

mod m20180508_101204_create_alerts;
mod m20180508_101216_create_guardians;
mod m20180508_101223_create_activities;
mod m20180508_101240_create_planned_activities;
mod m20180508_101326_populate_activities;
mod m20180905_090102_populate_activities;
mod m20180921_110336_add_admins;
mod m20180922_104727_add_superadmins;
mod m20250828_224244_add_destiny_rising_uids;
mod m20250910_152923_rename_tables;
mod tables;

pub use tables::*;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migration_table_name() -> DynIden {
        "__seaql_migrations".into_iden()
    }

    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20180508_101204_create_alerts::Migration),
            Box::new(m20180508_101216_create_guardians::Migration),
            Box::new(m20180508_101223_create_activities::Migration),
            Box::new(m20180508_101240_create_planned_activities::Migration),
            Box::new(m20180508_101326_populate_activities::Migration),
            Box::new(m20180905_090102_populate_activities::Migration),
            Box::new(m20180921_110336_add_admins::Migration),
            Box::new(m20180922_104727_add_superadmins::Migration),
            Box::new(m20250828_224244_add_destiny_rising_uids::Migration),
            Box::new(m20250910_152923_rename_tables::Migration),
        ]
    }
}
