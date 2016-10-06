package org.aeriagloris.persistence

import java.sql.*

class JdbcStore(val driverClass: String, val connectionString: String): UserStore, PlannedActivityStore {
    private val connection: Connection

    init {
        Class.forName(driverClass)
        connection = DriverManager.getConnection(connectionString)
    }

    override fun createUserRecord(userData: UserData): Int? {
        return connection.executeInsert("users",
                "telegram_name" to userData.telegramName,
                "telegram_id" to userData.telegramId,
                "psn_name" to userData.psnName)
                //"email" to userData.email,
                //"psn_clan" to userData.psnClan,
                //"created_at" to null,
                //"updated_at" to null,
                //"deleted_at" to null,
                //"tokens" to userData.tokens,
                //"pending_activation_code" to userData.pendingActivationCode
    }

//    override fun createFeed(data: FeedData): Int {
//        return connection.executeInsert("users",
//                "username" to data.userName,
//                "screen_name" to data.screenName,
//                "hashed_password" to data.hashedPassword,
//                "email" to data.email,
//                "is_private" to (if (data.private) 1 else 0),
//                "type" to (if (data.feedType == FeedType.Group) "group" else "user"),
//                "profile" to data.profile)
//    }
//
//    override fun loadFeed(id: Int): FeedData? {
//        return connection.executeQuery("select * from users where id=?", id) { rs ->
//            FeedData(feedTypeFromString(rs.getString("type")),
//                    rs.getString("username"),
//                    rs.getString("email"),
//                    rs.getString("hashed_password"),
//                    rs.getString("screen_name"),
//                    rs.getString("profile"),
//                    rs.getInt("is_private") != 0)
//        }
//    }
//
//    private fun feedTypeFromString(s: String): FeedType = when(s) {
//        "group" -> FeedType.Group
//        else -> FeedType.User
//    }
//
    override fun lookupTelegramUserName(telegramUserName: String): Int? {
        return connection.executeQuery("select id from users where telegram_name=?", telegramUserName) { it.getInt("id") }
    }

//    override fun createSubscription(fromUserId: Int, toFeedId: Int) {
//        connection.executeInsert("subscriptions", "subscriber_id" to fromUserId, "subscription_id" to toFeedId)
//    }
//
//    override fun removeSubscription(fromUserId: Int, toFeedId: Int) {
//        connection.executeUpdate("delete from subscriptions where subscriber_id=? and subscription_id=?",
//                fromUserId, toFeedId)
//    }
//
//    override fun loadSubscriptions(userId: Int): List<Int> {
//        return connection.executeListQuery(
//                "select subscription_id from subscriptions where subscriber_id=?", userId) { it.getInt("subscription_id") }
//    }
//
//    override fun loadSubscribers(feedId: Int): List<Int> {
//        return connection.executeListQuery(
//                "select subscriber_id from subscriptions where subscription_id=?", feedId) { it.getInt("subscriber_id") }
//    }
//
//    override fun createBlock(fromUserId: Int, toUserId: Int) {
//        connection.executeInsert("blocks", "blocker_id" to fromUserId, "target_id" to toUserId)
//    }
//
//    override fun removeBlock(fromUserId: Int, toUserId: Int) {
//        connection.executeUpdate("delete from blocks where blocker_id=? and target_id=?",
//                fromUserId, toUserId)
//    }
//
//    override fun loadBlocks(userId: Int): List<Int> {
//        return connection.executeListQuery(
//                "select target_id from blocks where blocker_id=?", userId) { it.getInt("target_id") }
//    }
//
//    override fun createAdmin(groupId: Int, adminId: Int) {
//        connection.executeInsert("group_admins", "group_id" to groupId, "admin_id" to adminId)
//    }
//
//    override fun removeAdmin(groupId: Int, adminId: Int) {
//        connection.executeUpdate("delete from group_admins where group_id=? and admin_id=?",
//                groupId, adminId)
//    }
//
//    override fun loadAdmins(groupId: Int): List<Int> {
//        return connection.executeListQuery(
//                "select admin_id from group_admins where group_id=?", groupId) { it.getInt("admin_id") }
//    }
//
//    override fun createSubscriptionRequest(fromUserId: Int, toUserId: Int) {
//        connection.executeInsert("subscription_requests", "subscriber_id" to fromUserId, "target_id" to toUserId)
//    }
//
//    override fun removeSubscriptionRequest(fromUserId: Int, toUserId: Int) {
//        connection.executeUpdate("delete from subscription_requests where subscriber_id=? and target_id=?",
//                fromUserId, toUserId)
//    }
//
//    override fun loadSubscriptionRequests(targetUser: Int): List<Int> {
//        return connection.executeListQuery(
//                "select subscriber_id from subscription_requests where target_id=? order by created_at desc", targetUser) { it.getInt("subscriber_id")}
//    }
//
    override fun createPlannedActivity(data: PlannedActivityData): Int {
        return connection.executeInsert("planned_activity",
                "author_id" to data.authorId,
                "activity_id" to data.activityId,
                "details" to data.details,
                "start" to Date(data.start))
    }

//    override fun updatePost(id: Int, data: PostData) {
//        connection.executeUpdate("update posts set updated_at=? where id=?", Date(data.updatedAt), id)
//    }
//
//    override fun createLike(userId: Int, postId: Int, timestamp: Long) {
//        connection.executeInsert("likes",
//                "user_id" to userId,
//                "post_id" to postId,
//                "created_at" to Date(timestamp))
//    }
//
//    override fun removeLike(userId: Int, postId: Int) {
//        connection.executeUpdate("delete from likes where user_id=? and post_id=?", userId, postId)
//    }
//
//    override fun deletePostWithLikes(postId: Int) {
//        connection.executeUpdate("delete from comments where post_id=?", postId)
//        connection.executeUpdate("delete from likes where post_id=?", postId)
//        connection.executeUpdate("delete from posts where id=?", postId)
//    }
//
//    override fun createComment(commentData: CommentData): Int {
//        return connection.executeInsert("comments",
//                "user_id" to commentData.author,
//                "post_id" to commentData.postId,
//                "body" to commentData.body,
//                "created_at" to Date(commentData.createdAt))
//    }
//
//    override fun deleteComment(commentId: Int) {
//        connection.executeUpdate("delete from comments where id=?", commentId)
//    }
//
//    override fun loadUserPostIds(author: Int): List<Int> {
//        return connection.executeListQuery("select id from posts where user_id=?", author) { it.getInt("id") }
//    }
//
//    override fun loadPost(postId: Int): PostData? {
//        return connection.executeQuery("select * from posts where id=?", postId) { rs ->
//            PostData(rs.getDate("created_at").time,
//                    rs.getDate("updated_at").time,
//                    rs.getInt("user_id"),
//                    intArrayOf(rs.getInt("user_id")),
//                    rs.getString("body"))
//        }
//    }
//
//    override fun loadLikesSortedByTimestamp(postId: Int): List<Int> {
//        return connection.executeListQuery("select user_id from likes where post_id=? order by created_at desc", postId) { it.getInt("user_id")}
//    }
//
//    override fun loadComments(postId: Int): List<Pair<Int, CommentData>> {
//        return connection.executeListQuery("select * from comments where post_id=?", postId) { rs ->
//            rs.getInt("id") to CommentData(postId,
//                    rs.getDate("created_at").time,
//                    rs.getInt("user_id"),
//                    rs.getString("body"))
//        }
//    }
//
//    override fun loadUserLikesSortedByTimestamp(userId: Int): List<Int> {
//        return connection.executeListQuery("select post_id from likes where user_id=? order by created_at desc", userId) {
//            it.getInt("post_id")
//        }
//    }
//
//    override fun loadUserCommentedPosts(userId: Int): List<Int> {
//        return connection.executeListQuery("select distinct post_id, created_at from comments where user_id=? order by created_at desc", userId) {
//            it.getInt("post_id")
//        }
//    }
}
