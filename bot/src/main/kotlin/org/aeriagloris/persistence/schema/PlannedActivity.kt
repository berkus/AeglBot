package org.aeriagloris.persistence.schema

import org.jetbrains.exposed.sql.*
import org.jetbrains.exposed.dao.*

object PlannedActivities : IntIdTable() {
    val authorId = reference("author_id", Guardians)
    val activityId = reference("activity_id", Activities)
    val details = text("details").nullable()
    val start = datetime("start")
}

class PlannedActivity(id: EntityID<Int>) : IntEntity(id) {
    companion object : IntEntityClass<PlannedActivity>(PlannedActivities)

    var author by Guardian referencedOn PlannedActivities.authorId
    var activity by Activity referencedOn PlannedActivities.activityId
    var start by PlannedActivities.start
    var details by PlannedActivities.details

    val members by PlannedActivityMember referrersOn PlannedActivityMembers.plannedActivityId

    fun joinLink(): String = "/join "+id

    fun membersFormatted(): String = members.toList().joinToString { memb -> memb.user.formatName() }
}
