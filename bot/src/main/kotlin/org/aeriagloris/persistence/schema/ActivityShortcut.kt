package org.aeriagloris.persistence.schema

import org.jetbrains.exposed.sql.*
import org.jetbrains.exposed.dao.*

object ActivityShortcuts : IntIdTable() { // @todo: StringIdTable when/if available
    val name = text("name").uniqueIndex()
    val game = text("game")
    val link = reference("link", Activities)
}

class ActivityShortcut(id: EntityID<Int>) : IntEntity(id) {
    companion object : IntEntityClass<ActivityShortcut>(ActivityShortcuts)

    var name by ActivityShortcuts.name
    var game by ActivityShortcuts.game
    var link by Activity referencedOn ActivityShortcuts.link
}
