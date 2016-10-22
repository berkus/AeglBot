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
                }
            }
        }
    }

//    override fun lookupTelegramUserName(telegramUserName: String): Int? {
//        return transaction {
//            Users.select { Users.telegramName eq telegramUserName }.single()[Users.id]
//        }
//    }
//
//    override fun loadUserData(id: Int): UserData? {
//        val user = transaction { Users.select { Users.id eq id }.single() }
//        return UserData(
//            telegramName = user[Users.telegramName],
//            telegramId = user[Users.telegramId],
//            psnName = user[Users.psnName],
//            email = user[Users.email],
//            psnClan = user[Users.psnClan],
//            createdAt = user[Users.createdAt],
//            updatedAt = user[Users.updatedAt],
//            deletedAt = user[Users.deletedAt],
//            //tokens = (user[Users.tokens] ? JSONObject(user[Users.tokens]) : null),
//            pendingActivationCode = user[Users.pendingActivationCode]
//        )
//    }
//
//    override fun createPlannedActivity(data: PlannedActivityData): Int? {
//        return transaction { 
//            PlannedActivity.insert {
//                it[authorId] = data.authorId
//                it[activityId] = data.activityId
//                it[details] = data.details
//                it[start] = data.start
//            } get PlannedActivity.id
//        }
//    }

//    override fun addUserToPlannedActivity(user: Int, activity: Int): Bool {
//        return transaction {
//            PlannedActivityMembers.insert {
//                it[activityId] = activity
//                it[userId] = user
//            }
//        }
//    }
//
//    override fun loadActivityData(id: Int): ActivityData? {
//        val row = transaction { Activity.select { Activity.id eq id }.single() }
//        return ActivityData(
//            id = row[Activity.id],
//            name = row[Activity.name],
//            mode = row[Activity.mode],
//            minFireteamSize = row[Activity.minFireteamSize],
//            maxFireteamSize = row[Activity.maxFireteamSize],
//            minLight = row[Activity.minLight],
//            minLevel = row[Activity.minLevel]
//        )
//    }
//
//    // Lookup activity using short code
//    override fun lookupActivity(code: String): Int? {
//        val lookup = Activities.map[code]
//        if (lookup == null) {
//            return null
//        }
//
//        return transaction {
//            Activity.select {
//                Activity.name.eq(lookup.first) and Activity.mode.eq(lookup.second)
//            }.single()[Activity.id]
//        }
//    }
}
