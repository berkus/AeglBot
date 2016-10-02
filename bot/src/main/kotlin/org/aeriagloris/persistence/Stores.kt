package org.aeriagloris.persistence

interface UserStore {
    fun lookupTelegramUserName(telegramUserName: String): Int?
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
