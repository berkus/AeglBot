package org.aeriagloris.persistence.schema

import org.jetbrains.exposed.sql.*
import org.jetbrains.exposed.dao.*

object PlannedActivityReminders : IntIdTable() {
    val plannedActivityId = reference("planned_activity_id", PlannedActivities)
    val userId = reference("user_id", Users)
    val remind = datetime("remind")
}

class PlannedActivityReminder(id: EntityID<Int>) : IntEntity(id) {
    companion object : IntEntityClass<PlannedActivityReminder>(PlannedActivityReminders)

    var user by User referencedOn PlannedActivityReminders.userId
    var activity by PlannedActivity referencedOn PlannedActivityReminders.plannedActivityId
    var reminder by PlannedActivityReminders.remind
}
