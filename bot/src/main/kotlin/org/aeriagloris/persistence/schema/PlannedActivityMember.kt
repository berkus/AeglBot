package org.aeriagloris.persistence.schema

import org.jetbrains.exposed.sql.*
import org.jetbrains.exposed.dao.*
import org.joda.time.DateTime

object PlannedActivityMembers : IntIdTable() {
    val plannedActivityId = reference("planned_activity_id", PlannedActivities)
    val userId = reference("user_id", Users)
    val added = datetime("added").default(DateTime.now())

    init {
        index(true, plannedActivityId, userId)
    }
}

class PlannedActivityMember(id: EntityID<Int>) : IntEntity(id) {
    companion object : IntEntityClass<PlannedActivityMember>(PlannedActivityMembers)

    var user by User referencedOn PlannedActivityMembers.userId
    var activity by PlannedActivity referencedOn PlannedActivityMembers.plannedActivityId
    var added by PlannedActivityMembers.added
}
