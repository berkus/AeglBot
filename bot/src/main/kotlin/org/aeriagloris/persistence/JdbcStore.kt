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
            logger.addLogger(StdOutSqlLogger())

            create(Guardians, Activities, PlannedActivities, PlannedActivityMembers, PlannedActivityReminders)

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
                        minFireteamSize = 1; maxFireteamSize = 12
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
                }
            }
        }
    }
}
