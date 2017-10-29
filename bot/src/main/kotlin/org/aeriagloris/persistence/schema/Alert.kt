package org.aeriagloris.persistence.schema

import org.jetbrains.exposed.sql.*
import org.jetbrains.exposed.dao.*

object Alerts : IntIdTable() {
    val guid = text("guid").uniqueIndex()
    val title = text("title")
    val type = text("type")
    val startDate = datetime("startDate")
    val expiryDate = datetime("expiryDate").nullable()
    val faction = text("faction").nullable()
}

class Alert(id: EntityID<Int>) : IntEntity(id) {
    companion object : IntEntityClass<Alert>(Alerts)

    var guid by Alerts.guid
    var title by Alerts.title
    var type by Alerts.type
    var startDate by Alerts.startDate
    var expiryDate by Alerts.expiryDate
    var faction by Alerts.faction
}
