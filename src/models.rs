// use super::schema::*;
use chrono::NaiveDateTime;
use serde_json::Value;

#[derive(Queryable)]
pub struct ActivityShortcut {
    pub id: i32,
    pub name: String,
    pub game: String,
    pub link: i32,
}

// #[derive(Insertable)]
// #[table_name = "activityshortcuts"]

// object Activities : IntIdTable() {
//     val name = text("name").index(false)
//     val mode = text("mode").nullable()
//     val minFireteamSize = integer("min_fireteam_size")
//     val maxFireteamSize = integer("max_fireteam_size")
//     val minLight = integer("min_light").nullable()
//     val minLevel = integer("min_level").nullable()
// }

// class Activity(id: EntityID<Int>) : IntEntity(id) {
//     companion object : IntEntityClass<Activity>(Activities)

//     var name by Activities.name
//     var mode by Activities.mode
//     var minFireteamSize by Activities.minFireteamSize
//     var maxFireteamSize by Activities.maxFireteamSize
//     var minLight by Activities.minLight
//     var minLevel by Activities.minLevel

//     fun formatName(): String = name + " " + mode
// }

#[derive(Queryable)]
// #[table_name = "activities"]
pub struct Activity {
    pub id: i32,
}

// object Alerts : IntIdTable() {
//     val guid = text("guid").uniqueIndex()
//     val title = text("title")
//     val type = text("type")
//     val startDate = datetime("startDate")
//     val expiryDate = datetime("expiryDate").nullable()
//     val faction = text("faction").nullable()
// }

// class Alert(id: EntityID<Int>) : IntEntity(id) {
//     companion object : IntEntityClass<Alert>(Alerts)

//     var guid by Alerts.guid
//     var title by Alerts.title
//     var type by Alerts.type
//     var startDate by Alerts.startDate
//     var expiryDate by Alerts.expiryDate
//     var faction by Alerts.faction
// }

#[derive(Queryable)]
// #[table_name = "alerts"]
pub struct Alert {
    pub id: i32,
}

// object Guardians : IntIdTable() {
//     val email = text("email").nullable()
//     val psnClan = text("psn_clan").nullable()
//     val createdAt = datetime("created_at").default(DateTime.now())
//     val updatedAt = datetime("updated_at").default(DateTime.now())
//     val deletedAt = datetime("deleted_at").nullable()
//     val tokens = text("tokens").nullable() // Should be `jsonb` actually...
//     val pendingActivationCode = text("pending_activation_code").nullable()
// }

#[derive(Queryable)]
// #[table_name = "guardians"]
pub struct Guardian {
    pub id: i32,
    pub telegram_name: String,
    pub telegram_id: i32,
    pub psn_name: String,
    pub email: Option<String>,
    pub psn_clan: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub tokens: Option<Value>,
    pub pending_activation_code: Option<String>,
}

impl Guardian {
    pub fn format_name(&self) -> String {
        format!("{} (t.me/{})", self.psn_name, self.telegram_name)
    }
}

// object PlannedActivities : IntIdTable() {
//     val authorId = reference("author_id", Guardians)
//     val activityId = reference("activity_id", Activities)
//     val details = text("details").nullable()
//     val start = datetime("start")
// }

// class PlannedActivity(id: EntityID<Int>) : IntEntity(id) {
//     companion object : IntEntityClass<PlannedActivity>(PlannedActivities)

//     var author by Guardian referencedOn PlannedActivities.authorId
//     var activity by Activity referencedOn PlannedActivities.activityId
//     var start by PlannedActivities.start
//     var details by PlannedActivities.details

//     val members by PlannedActivityMember referrersOn PlannedActivityMembers.plannedActivityId

//     fun joinLink(): String = "/join $id"

//     fun membersFormatted(joiner: String): String = members.toList().joinToString(joiner) { it.user.formatName() }

//     fun membersFormattedList(): String = membersFormatted(", ")

//     fun membersFormattedColumn(): String = membersFormatted("\n")

//     fun requiresMoreMembers(): Boolean = members.count() < activity.minFireteamSize

//     fun isFull(): Boolean = members.count() >= activity.maxFireteamSize

//     fun detailsFormatted(): String = if ("".equals(details)) { "" } else { details + "\n" }

//     fun joinPrompt(): String = if (isFull()) {
//             "This activity fireteam is full."
//         } else {
//             val count = activity.maxFireteamSize - members.count()
//             "Enter `${joinLink()}` to join this group. Up to $count more can join."
//         }
// }

#[derive(Queryable)]
// #[table_name = "plannedactivities"]
pub struct PlannedActivity {
    pub id: i32,
}

// object PlannedActivityMembers : IntIdTable() {
//     val plannedActivityId = reference("planned_activity_id", PlannedActivities)
//     val userId = reference("user_id", Guardians)
//     val added = datetime("added").default(DateTime.now())

//     init {
//         index(true, plannedActivityId, userId)
//     }
// }

// class PlannedActivityMember(id: EntityID<Int>) : IntEntity(id) {
//     companion object : IntEntityClass<PlannedActivityMember>(PlannedActivityMembers)

//     var user by Guardian referencedOn PlannedActivityMembers.userId
//     var activity by PlannedActivity referencedOn PlannedActivityMembers.plannedActivityId
//     var added by PlannedActivityMembers.added
// }

#[derive(Queryable)]
// #[table_name = "plannedactivitymembers"]
pub struct PlannedActivityMember {
    pub id: i32,
}

// object PlannedActivityReminders : IntIdTable() {
//     val plannedActivityId = reference("planned_activity_id", PlannedActivities)
//     val userId = reference("user_id", Guardians)
//     val remind = datetime("remind")
// }

// class PlannedActivityReminder(id: EntityID<Int>) : IntEntity(id) {
//     companion object : IntEntityClass<PlannedActivityReminder>(PlannedActivityReminders)

//     var user by Guardian referencedOn PlannedActivityReminders.userId
//     var activity by PlannedActivity referencedOn PlannedActivityReminders.plannedActivityId
//     var reminder by PlannedActivityReminders.remind
// }

#[derive(Queryable)]
// #[table_name = "plannedactivityreminders"]
pub struct PlannedActivityReminder {
    pub id: i32,
}
