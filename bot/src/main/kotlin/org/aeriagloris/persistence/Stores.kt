package org.aeriagloris.persistence

import org.json.JSONObject

data class UserData(
    val telegramName: String,
    val telegramId: Int,
    val psnName: String,
    val email: String? = null,
    val psnClan: String? = null,
    val createdAt: Long? = null,
    val updatedAt: Long? = null,
    val deletedAt: Long? = null,
    val tokens: JSONObject = JSONObject(),
    val pendingActivationCode: String? = null)

interface UserStore {
    fun lookupTelegramUserName(telegramUserName: String): Int?
    fun createUserRecord(userData: UserData): Int?
}

data class PlannedActivityData(
    val authorId: Int,
    val activityId: Int,
    var start: Long,
    val details: String)

//data class CommentData(val postId: Int, val createdAt: Long, val author: Int, val body: String)

interface PlannedActivityStore {
    fun createPlannedActivity(data: PlannedActivityData): Int
}
