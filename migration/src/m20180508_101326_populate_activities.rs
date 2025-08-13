use {
    crate::{Activities, ActivityShortcuts},
    sea_orm_migration::{prelude::*, sea_orm::TransactionTrait},
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
            (1,  "Vault of Glass",             "normal",              1, 6,  Some(200), Some(25)),
            (2,  "Vault of Glass",             "hard",                1, 6,  Some(280), Some(30)),
            (3,  "Crota's End",                "normal",              1, 6,  Some(200), Some(30)),
            (4,  "Crota's End",                "hard",                1, 6,  Some(280), Some(33)),
            (5,  "King's Fall",                "normal",              1, 6,  Some(290), Some(35)),
            (6,  "King's Fall",                "hard",                1, 6,  Some(320), Some(40)),
            (7,  "Wrath of the Machine",       "normal",              1, 6,  Some(360), Some(40)),
            (8,  "Wrath of the Machine",       "hard",                1, 6,  Some(380), Some(40)),
            (9,  "Vanguard",                   "Patrols",             1, 3,  None,      None),
            (10, "Vanguard",                   "Archon's Forge",      1, 3,  None,      None),
            (11, "Vanguard",                   "Court of Oryx",       1, 3,  None,      None),
            (12, "Vanguard",                   "any",                 1, 3,  None,      None),
            (13, "Crucible",                   "Private Matches",     1, 12, None,      None),
            (14, "Crucible",                   "Trials of Osiris",    3, 3,  Some(370), Some(40)),
            (15, "Crucible",                   "Iron Banner",         1, 6,  Some(350), Some(40)),
            (16, "Crucible",                   "6v6",                 1, 6,  None,      None),
            (17, "Crucible",                   "3v3",                 1, 3,  None,      None),
            (19, "Crucible",                   "Private Tournament",  1, 12, None,      None),
            (20, "Vanguard",                   "Challenge of Elders", 1, 3,  Some(320), None),
            (21, "Vanguard",                   "Prison of Elders",    1, 3,  None,      None),
            (22, "Vanguard",                   "Nightfall",           1, 3,  Some(380), None),
            (18, "Crucible",                   "any",                 1, 6,  None,      None),
            (23, "Dolmen",                     "any",                 1, 12, None,      Some(1)),
            (25, "Dungeon",                    "any",                 3, 12, None,      Some(8)),
            (26, "Questing",                   "any",                 1, 12, None,      Some(1)),
            (27, "Cyrodiil",                   "pvp",                 1, 12, None,      Some(10)),
            (24, "Delve",                      "any",                 1, 4,  None,      Some(1)),
            (28, "Alienation",                 "coop",                1, 4,  None,      Some(1)),
            (29, "Crucible",                   "4v4",                 1, 4,  None,      None),
            (30, "Titanfall 2",                "coop",                1, 4,  None,      None),
            (31, "Titanfall 2",                "pvp",                 1, 6,  None,      None),
            (32, "Leviathan",                  "normal",              1, 6,  Some(100), Some(15)),
            (33, "Crucible",                   "Trials of the Nine",  4, 4,  Some(250), Some(20)),
            (34, "Warframe",                   "pve",                 1, 4,  None,      None),
            (35, "Warframe",                   "pvp",                 1, 4,  None,      None),
            (36, "Warframe",                   "Index",               1, 4,  None,      None),
            (37, "Warframe",                   "Raid (obsolete)",     4, 8,  None,      None),
            (38, "Leviathan",                  "prestige",            1, 6,  Some(300), Some(20)),
            (39, "Leviathan, Eater of Worlds", "normal",              1, 6,  Some(300), Some(20)),
            (40, "Leviathan, Eater of Worlds", "prestige",            1, 6,  Some(315), Some(25)),
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
                    act.5.into(), //.unwrap_or(Expr::value("null")).into(),
                    act.6.into(), // unwrap_or(Expr::value("null")).into(),
                ])
                .to_owned();

            let insert = builder.build(&insert);
            transaction.execute(insert).await?;
        }

        // INSERT INTO activityshortcuts (id, name, game, link)
        // VALUES
        #[rustfmt::skip]
        let shortcuts = vec![
            (1,  "kf",      "Destiny",     6),
            (2,  "kfh",     "Destiny",     6),
            (3,  "kfn",     "Destiny",     5),
            (4,  "cr",      "Destiny",     4),
            (5,  "crh",     "Destiny",     4),
            (6,  "crn",     "Destiny",     3),
            (7,  "vog",     "Destiny",     2),
            (8,  "vogh",    "Destiny",     2),
            (9,  "vogn",    "Destiny",     1),
            (10, "wotm",    "Destiny",     7),
            (11, "wotmh",   "Destiny",     8),
            (12, "wotmn",   "Destiny",     7),
            (13, "pvp",     "Destiny",     18),
            (14, "3v3",     "Destiny",     17),
            (15, "6v6",     "Destiny",     16),
            (16, "ib",      "Destiny",     15),
            (17, "too",     "Destiny",     14),
            (18, "pvt",     "Destiny",     13),
            (19, "trn",     "Destiny",     19),
            (20, "pve",     "Destiny",     12),
            (21, "patrol",  "Destiny",     9),
            (22, "coo",     "Destiny",     11),
            (23, "forge",   "Destiny",     10),
            (24, "poe",     "Destiny",     21),
            (25, "coe",     "Destiny",     20),
            (26, "nf",      "Destiny",     22),
            (27, "dolmen",  "TESO",        23),
            (28, "delve",   "TESO",        24),
            (29, "dung",    "TESO",        25),
            (30, "quest",   "TESO",        26),
            (31, "cyro",    "TESO",        27),
            (32, "cyrod",   "TESO",        27),
            (33, "alien",   "Alienation",  28),
            (34, "pvp2",    "Destiny 2",   29),
            (35, "tf2coop", "Titanfall 2", 30),
            (36, "tf2pvp",  "Titanfall 2", 31),
            (37, "levin",   "Destiny 2",   32),
            (38, "to9",     "Destiny 2",   33),
            (39, "wfpve",   "Warframe",    34),
            (40, "wfpvp",   "Warframe",    35),
            (41, "wfindex", "Warframe",    36),
            (42, "wfraid",  "Warframe",    37),
            (43, "levip",   "Destiny 2",   38),
            (44, "eaten",   "Destiny 2",   39),
            (45, "eatep",   "Destiny 2",   40),
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
            .execute_unprepared("DELETE FROM activityshortcuts WHERE id BETWEEN 1 AND 45")
            .await?;
        transaction
            .execute_unprepared("DELETE FROM activities WHERE id BETWEEN 1 AND 40")
            .await?;

        transaction.commit().await?;

        Ok(())
    }
}
