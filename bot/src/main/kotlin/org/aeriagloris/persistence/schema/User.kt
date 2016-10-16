package org.aeriagloris.persistence.schema

import org.jetbrains.exposed.sql.*
import org.jetbrains.exposed.dao.*
import org.joda.time.DateTime

object Users : IntIdTable() {
    val telegramName = text("telegram_name").uniqueIndex()
    val telegramId = integer("telegram_id").uniqueIndex()
    val psnName = text("psn_name").uniqueIndex()
    val email = text("email").nullable()
    val psnClan = text("psn_clan").nullable()
    val createdAt = datetime("created_at").default(DateTime.now())
    val updatedAt = datetime("updated_at").default(DateTime.now())
    val deletedAt = datetime("deleted_at").nullable()
    val tokens = text("tokens").nullable() // Should be `jsonb` actually...
    val pendingActivationCode = text("pending_activation_code").nullable()
}

class User(id: EntityID<Int>) : IntEntity(id) {
    companion object : IntEntityClass<User>(Users)

    var telegramName by Users.telegramName
    var telegramId by Users.telegramId
    var psnName by Users.psnName

    // Synthetics
    //val ownedActivities by Activity backReferenceOn PlannedActivities.authorId
    //val allActivities by Activity optionalReferrersOn PlannedActivityMembers.userId
}
