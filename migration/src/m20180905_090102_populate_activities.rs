use {
    crate::{Activities, ActivityShortcuts},
    sea_orm_migration::{
        prelude::*,
        sea_orm::{TransactionTrait, Value},
    },
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // @fixme: this already runs in a transaction, no need for a new one...
        let transaction = manager.get_connection().begin().await?;
        let builder = transaction.get_database_backend();

        // INSERT INTO activities (id, name, mode, min_fireteam_size, max_fireteam_size, min_light, min_level)
        // VALUES
        #[rustfmt::skip]
        let activities = vec![
            (41, "Vanguard",                  "Escalation Protocol", 1, 9, Some(350), None),
            (42, "Leviathan, Spire of Stars", "normal",              6, 6, Some(370), Some(30)),
            (43, "Leviathan, Spire of Stars", "prestige",            6, 6, Some(385), Some(30)),
            (44, "King's Fall",               "weekly",              6, 6, Some(390), Some(40)),
            (45, "Crota's End",               "weekly",              6, 6, Some(390), Some(40)),
            (46, "Vault of Glass",            "weekly",              6, 6, Some(390), Some(40)),
            (47, "Wrath of the Machine",      "weekly",              6, 6, Some(390), Some(40)),
            (48, "Last Wish",                 "normal",              6, 6, Some(450), Some(40)),
            (49, "Last Wish",                 "prestige",            6, 6, Some(500), Some(40)),
            (50, "Gambit",                    "pve/pvp",             1, 4, Some(400), Some(30)),
        ];
        for act in activities {
            let insert = Query::insert()
                .into_table(Activities::Table)
                .columns([
                    Activities::Id,
                    Activities::Name,
                    Activities::Mode,
                    Activities::MinFireteamSize,
                    Activities::MaxFireteamSize,
                    Activities::MinLight,
                    Activities::MinLevel,
                ])
                .values_panic([
                    act.0.into(),
                    act.1.into(),
                    act.2.into(),
                    act.3.into(),
                    act.4.into(),
                    act.5
                        .map(|v| v.into())
                        .unwrap_or(Expr::value(Value::Int(None))),
                    act.6
                        .map(|v| v.into())
                        .unwrap_or(Expr::value(Value::Int(None))),
                ])
                .to_owned();

            let insert = builder.build(&insert);
            transaction.execute(insert).await?;
        }

        // INSERT INTO activityshortcuts (id, name, game, link)
        // VALUES
        #[rustfmt::skip]
        let shortcuts = vec![
            (46, "escal8", "Destiny 2", 41),
            (47, "spiren", "Destiny 2", 42),
            (48, "spirep", "Destiny 2", 43),
            (49, "kfw",    "Destiny",   44),
            (50, "crw",    "Destiny",   45),
            (51, "vogw",   "Destiny",   46),
            (52, "wotmw",  "Destiny",   47),
            (53, "lastwn", "Destiny 2", 48),
            (54, "lastwp", "Destiny 2", 49),
            (55, "gambit", "Destiny 2", 50),
        ];
        for shr in shortcuts {
            let insert = Query::insert()
                .into_table(ActivityShortcuts::Table)
                .columns([
                    ActivityShortcuts::Id,
                    ActivityShortcuts::Name,
                    ActivityShortcuts::Game,
                    ActivityShortcuts::Link,
                ])
                .values_panic([shr.0.into(), shr.1.into(), shr.2.into(), shr.3.into()])
                .to_owned();

            let insert = builder.build(&insert);
            transaction.execute(insert).await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let transaction = manager.get_connection().begin().await?;

        transaction
            .execute_unprepared("DELETE FROM activityshortcuts WHERE id BETWEEN 46 AND 55")
            .await?;
        transaction
            .execute_unprepared("DELETE FROM activities WHERE id BETWEEN 41 AND 50")
            .await?;

        transaction.commit().await?;

        Ok(())
    }
}
