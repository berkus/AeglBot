package org.aeriagloris.persistence

import org.json.JSONObject
import org.joda.time.DateTime
import org.jetbrains.exposed.sql.*
import org.jetbrains.exposed.sql.transactions.transaction
import org.jetbrains.exposed.sql.SchemaUtils.create
import org.jetbrains.exposed.dao.*
import org.aeriagloris.persistence.schema.*

class JdbcStore(val driverClass: String, val connectionString: String) {
    init {
        Database.connect(connectionString, driver = driverClass)

        transaction {
            logger.addLogger(Slf4jSqlLogger())

            create(Alerts, Guardians, Activities, ActivityShortcuts, PlannedActivities, PlannedActivityMembers, PlannedActivityReminders)

            if (Activity.count() == 0) {
                transaction {
                    Activity.new {
                        name = "Vault of Glass"; mode = "normal"
                        minFireteamSize = 1; maxFireteamSize = 6
                        minLight = 200; minLevel = 25
                    }
                    Activity.new {
                        name = "Vault of Glass"; mode = "hard"
                        minFireteamSize = 1; maxFireteamSize = 6
                        minLight = 280; minLevel = 30
                    }
                    Activity.new {
                        name = "Crota's End"; mode = "normal"
                        minFireteamSize = 1; maxFireteamSize = 6
                        minLight = 200; minLevel = 30
                    }
                    Activity.new {
                        name = "Crota's End"; mode = "hard"
                        minFireteamSize = 1; maxFireteamSize = 6
                        minLight = 280; minLevel = 33
                    }
                    Activity.new {
                        name = "King's Fall"; mode = "normal"
                        minFireteamSize = 1; maxFireteamSize = 6
                        minLight = 290; minLevel = 35
                    }
                    Activity.new {
                        name = "King's Fall"; mode = "hard"
                        minFireteamSize = 1; maxFireteamSize = 6
                        minLight = 320; minLevel = 40
                    }
                    Activity.new {
                        name = "Wrath of the Machine"; mode = "normal"
                        minFireteamSize = 1; maxFireteamSize = 6
                        minLight = 360; minLevel = 40
                    }
                    Activity.new {
                        name = "Wrath of the Machine"; mode = "hard"
                        minFireteamSize = 1; maxFireteamSize = 6
                        minLight = 380; minLevel = 40
                    }
                    Activity.new {
                        name = "Vanguard"; mode = "Patrols"
                        minFireteamSize = 1; maxFireteamSize = 3
                    }
                    Activity.new {
                        name = "Vanguard"; mode = "Archon's Forge"
                        minFireteamSize = 1; maxFireteamSize = 3
                    }
                    Activity.new {
                        name = "Vanguard"; mode = "Court of Oryx"
                        minFireteamSize = 1; maxFireteamSize = 3
                    }
                    Activity.new {
                        name = "Vanguard"; mode = "any"
                        minFireteamSize = 1; maxFireteamSize = 3
                    }
                    Activity.new {
                        name = "Crucible"; mode = "Private Matches"
                        minFireteamSize = 1; maxFireteamSize = 12
                    }
                    Activity.new {
                        name = "Crucible"; mode = "Trials of Osiris"
                        minFireteamSize = 3; maxFireteamSize = 3
                        minLight = 370; minLevel = 40
                    }
                    Activity.new {
                        name = "Crucible"; mode = "Iron Banner"
                        minFireteamSize = 1; maxFireteamSize = 6
                        minLight = 350; minLevel = 40
                    }
                    Activity.new {
                        name = "Crucible"; mode = "6v6"
                        minFireteamSize = 1; maxFireteamSize = 6
                    }
                    Activity.new {
                        name = "Crucible"; mode = "3v3"
                        minFireteamSize = 1; maxFireteamSize = 3
                    }
                    Activity.new {
                        name = "Crucible"; mode = "any"
                        minFireteamSize = 1; maxFireteamSize = 6
                    }
                    Activity.new {
                        name = "Crucible"; mode = "Private Tournament"
                        minFireteamSize = 1; maxFireteamSize = 12
                    }
                    Activity.new {
                        name = "Vanguard"; mode = "Challenge of Elders"
                        minFireteamSize = 1; maxFireteamSize = 3
                        minLight = 320
                    }
                    Activity.new {
                        name = "Vanguard"; mode = "Prison of Elders"
                        minFireteamSize = 1; maxFireteamSize = 3
                    }
                    Activity.new {
                        name = "Vanguard"; mode = "Nightfall"
                        minFireteamSize = 1; maxFireteamSize = 3
                        minLight = 380
                    }

                    // Destiny 2

                    Activity.new {
                        name = "Crucible"; mode = "4v4"
                        minFireteamSize = 1; maxFireteamSize = 4
                    }

                    Activity.new {
                        name = "Leviathan"; mode = "normal"
                        minFireteamSize = 1; maxFireteamSize = 6
                        minLight = 100; minLevel = 15
                    }

                    Activity.new {
                        name = "Crucible"; mode = "Trials of the Nine"
                        minFireteamSize = 4; maxFireteamSize = 4
                        minLight = 250; minLevel = 20
                    }

                    // TESO

                    Activity.new {
                        name = "Dolmen"; mode = "any"
                        minFireteamSize = 1; maxFireteamSize = 12
                        minLevel = 1
                    }

                    Activity.new {
                        name = "Delve"; mode = "any"
                        minFireteamSize = 1; maxFireteamSize = 4
                        minLevel = 1
                    }

                    Activity.new {
                        name = "Dungeon"; mode = "any"
                        minFireteamSize = 3; maxFireteamSize = 12
                        minLevel = 8
                    }

                    Activity.new {
                        name = "Questing"; mode = "any"
                        minFireteamSize = 1; maxFireteamSize = 12
                        minLevel = 1
                    }

                    Activity.new {
                        name = "Cyrodiil"; mode = "pvp"
                        minFireteamSize = 1; maxFireteamSize = 12
                        minLevel = 10
                    }

                    // Alienation

                    Activity.new {
                        name = "Alienation"; mode = "coop"
                        minFireteamSize = 1; maxFireteamSize = 4
                        minLevel = 1
                    }

                    // Titanfall 2

                    Activity.new {
                        name = "Titanfall 2"; mode = "coop"
                        minFireteamSize = 1; maxFireteamSize = 4
                    }

                    Activity.new {
                        name = "Titanfall 2"; mode = "pvp"
                        minFireteamSize = 1; maxFireteamSize = 6
                    }

                    // Warframe

                    Activity.new {
                        name = "Warframe"; mode = "pve"
                        minFireteamSize = 1; maxFireteamSize = 4
                    }

                    Activity.new {
                        name = "Warframe"; mode = "pvp"
                        minFireteamSize = 1; maxFireteamSize = 4
                    }


                }
            }

            if (ActivityShortcut.count() == 0) {
                transaction {
                    ActivityShortcut.new {
                        name = "kf"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "King's Fall") and (Activities.mode eq "hard") }.single()
                    }
                    ActivityShortcut.new {
                        name = "kfh"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "King's Fall") and (Activities.mode eq "hard") }.single()
                    }
                    ActivityShortcut.new {
                        name = "kfn"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "King's Fall") and (Activities.mode eq "normal") }.single()
                    }
                    ActivityShortcut.new {
                        name = "cr"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Crota's End") and (Activities.mode eq "hard") }.single()
                    }
                    ActivityShortcut.new {
                        name = "crh"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Crota's End") and (Activities.mode eq "hard") }.single()
                    }
                    ActivityShortcut.new {
                        name = "crn"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Crota's End") and (Activities.mode eq "normal") }.single()
                    }
                    ActivityShortcut.new {
                        name = "vog"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Vault of Glass") and (Activities.mode eq "hard") }.single()
                    }
                    ActivityShortcut.new {
                        name = "vogh"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Vault of Glass") and (Activities.mode eq "hard") }.single()
                    }
                    ActivityShortcut.new {
                        name = "vogn"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Vault of Glass") and (Activities.mode eq "normal") }.single()
                    }
                    ActivityShortcut.new {
                        name = "wotm"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Wrath of the Machine") and (Activities.mode eq "normal") }.single()
                    }
                    ActivityShortcut.new {
                        name = "wotmh"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Wrath of the Machine") and (Activities.mode eq "hard") }.single()
                    }
                    ActivityShortcut.new {
                        name = "wotmn"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Wrath of the Machine") and (Activities.mode eq "normal") }.single()
                    }
                    ActivityShortcut.new {
                        name = "pvp"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Crucible") and (Activities.mode eq "any") }.single()
                    }
                    ActivityShortcut.new {
                        name = "3v3"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Crucible") and (Activities.mode eq "3v3") }.single()
                    }
                    ActivityShortcut.new {
                        name = "6v6"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Crucible") and (Activities.mode eq "6v6") }.single()
                    }
                    ActivityShortcut.new {
                        name = "ib"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Crucible") and (Activities.mode eq "Iron Banner") }.single()
                    }
                    ActivityShortcut.new {
                        name = "too"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Crucible") and (Activities.mode eq "Trials of Osiris") }.single()
                    }
                    ActivityShortcut.new {
                        name = "pvt"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Crucible") and (Activities.mode eq "Private Matches") }.single()
                    }
                    ActivityShortcut.new {
                        name = "trn"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Crucible") and (Activities.mode eq "Private Tournament") }.single()
                    }
                    ActivityShortcut.new {
                        name = "pve"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Vanguard") and (Activities.mode eq "any") }.single()
                    }
                    ActivityShortcut.new {
                        name = "patrol"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Vanguard") and (Activities.mode eq "Patrols") }.single()
                    }
                    ActivityShortcut.new {
                        name = "coo"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Vanguard") and (Activities.mode eq "Court of Oryx") }.single()
                    }
                    ActivityShortcut.new {
                        name = "forge"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Vanguard") and (Activities.mode eq "Archon's Forge") }.single()
                    }
                    ActivityShortcut.new {
                        name = "poe"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Vanguard") and (Activities.mode eq "Prison of Elders") }.single()
                    }
                    ActivityShortcut.new {
                        name = "coe"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Vanguard") and (Activities.mode eq "Challenge of Elders") }.single()
                    }
                    ActivityShortcut.new {
                        name = "nf"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Vanguard") and (Activities.mode eq "Nightfall") }.single()
                    }

                    // Destiny 2

                    ActivityShortcut.new {
                        name = "pvp2"
                        game = "Destiny 2"
                        link = Activity.find { (Activities.name eq "Crucible") and (Activities.mode eq "4v4") }.single()
                    }

                    ActivityShortcut.new {
                        name = "to9"
                        game = "Destiny"
                        link = Activity.find { (Activities.name eq "Crucible") and (Activities.mode eq "Trials of the Nine") }.single()
                    }

                    ActivityShortcut.new {
                        name = "levin"
                        game = "Destiny 2"
                        link = Activity.find { (Activities.name eq "Leviathan") and (Activities.mode eq "normal") }.single()
                    }

                    // TESO

                    ActivityShortcut.new {
                        name = "dolmen"
                        game = "TESO"
                        link = Activity.find { (Activities.name eq "Dolmen") and (Activities.mode eq "any") }.single()
                    }
                    ActivityShortcut.new {
                        name = "delve"
                        game = "TESO"
                        link = Activity.find { (Activities.name eq "Delve") and (Activities.mode eq "any") }.single()
                    }
                    ActivityShortcut.new {
                        name = "dung"
                        game = "TESO"
                        link = Activity.find { (Activities.name eq "Dungeon") and (Activities.mode eq "any") }.single()
                    }
                    ActivityShortcut.new {
                        name = "quest"
                        game = "TESO"
                        link = Activity.find { (Activities.name eq "Questing") and (Activities.mode eq "any") }.single()
                    }
                    ActivityShortcut.new {
                        name = "cyro"
                        game = "TESO"
                        link = Activity.find { (Activities.name eq "Cyrodiil") and (Activities.mode eq "pvp") }.single()
                    }
                    ActivityShortcut.new {
                        name = "cyrod"
                        game = "TESO"
                        link = Activity.find { (Activities.name eq "Cyrodiil") and (Activities.mode eq "pvp") }.single()
                    }

                    // Alienation

                    ActivityShortcut.new {
                        name = "alien"
                        game = "Alienation"
                        link = Activity.find { (Activities.name eq "Alienation") and (Activities.mode eq "coop") }.single()
                    }

                    // Titanfall 2

                    ActivityShortcut.new {
                        name = "tf2coop"
                        game = "Titanfall 2"
                        link = Activity.find { (Activities.name eq "Titanfall 2") and (Activities.mode eq "coop") }.single()
                    }

                    ActivityShortcut.new {
                        name = "tf2pvp"
                        game = "Titanfall 2"
                        link = Activity.find { (Activities.name eq "Titanfall 2") and (Activities.mode eq "pvp") }.single()
                    }

                    // Warframe

                   ActivityShortcut.new {
                        name = "wfpve"
                        game = "Warframe"
                        link = Activity.find { (Activities.name eq "Warframe") and (Activities.mode eq "pve") }.single()
                    }

                    ActivityShortcut.new {
                        name = "wfpvp"
                        game = "Warframe"
                        link = Activity.find { (Activities.name eq "Warframe") and (Activities.mode eq "pvp") }.single()
                    }
                }
            }
        }
    }
}

