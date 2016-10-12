package org.aeriagloris.persistence.schema

import org.jetbrains.exposed.sql.*
import org.jetbrains.exposed.dao.*

object Activities : IntIdTable() {
    val name = text("name").index(false)
    val mode = text("mode").nullable()
    val minFireteamSize = integer("min_fireteam_size")
    val maxFireteamSize = integer("max_fireteam_size")
    val minLight = integer("min_light").nullable()
    val minLevel = integer("min_level").nullable()
}

class Activity(id: EntityID<Int>) : IntEntity(id) {
    companion object : IntEntityClass<Activity>(Activities)

    var name by Activities.name
    var mode by Activities.mode
    var minFireteamSize by Activities.minFireteamSize
    var maxFireteamSize by Activities.maxFireteamSize
    var minLight by Activities.minLight
    var minLevel by Activities.minLevel

}
