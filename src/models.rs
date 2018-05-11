use super::schema::*;
use chrono::NaiveDateTime;
use serde_json::Value;
use std::fmt;

//
// ActivityShortcut
//

#[derive(Debug, Queryable, Identifiable, AsChangeset, Associations, Model)]
#[table_name = "activityshortcuts"]
#[belongs_to(Activity, foreign_key = "link")]
pub struct ActivityShortcut {
    pub id: i32,
    pub name: String,
    pub game: String,
    pub link: i32,
}

//
// Activity
//

#[derive(Debug, Queryable, Identifiable, AsChangeset, Model)]
#[table_name = "activities"]
pub struct Activity {
    pub id: i32,
    pub name: String,
    pub mode: Option<String>,
    pub min_fireteam_size: i32,
    pub max_fireteam_size: i32,
    pub min_light: Option<i32>,
    pub min_level: Option<i32>,
}

impl Activity {
    pub fn format_name(&self) -> String {
        format!(
            "{} {}",
            self.name,
            match self.mode {
                None => "",
                Some(ref x) => &x,
            }
        )
    }
}

//
// Alert
//

#[derive(Debug, Queryable, Identifiable, AsChangeset, Model)]
pub struct Alert {
    pub id: i32,
    pub guid: String,
    pub title: String,
    #[column_name = "type_"]
    pub alert_type: String,
    #[column_name = "startdate"]
    pub start_date: NaiveDateTime,
    #[column_name = "expirydate"]
    pub expiry_date: Option<NaiveDateTime>,
    pub faction: Option<String>,
    pub flavor: Option<String>,
}

#[derive(Clone, Insertable, NewModel)]
#[table_name = "alerts"]
#[model(Alert)]
pub struct NewAlert<'a> {
    pub guid: &'a str,
    pub title: &'a str,
    #[column_name = "type_"]
    pub alert_type: Option<&'a str>,
    #[column_name = "startdate"]
    pub start_date: Option<NaiveDateTime>,
    #[column_name = "expirydate"]
    pub expiry_date: Option<NaiveDateTime>,
    pub faction: Option<&'a str>,
    pub flavor: Option<&'a str>,
}

impl fmt::Display for Alert {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "✊ Alert: {}", self.title)
    }
}

//
// Guardian
//

#[derive(Debug, Clone, Queryable, Identifiable, AsChangeset, Model)]
pub struct Guardian {
    pub id: i32,
    pub telegram_name: String,
    pub telegram_id: i64,
    pub psn_name: String,
    pub email: Option<String>,
    pub psn_clan: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub tokens: Option<Value>,
    pub pending_activation_code: Option<String>,
}

#[derive(Insertable, NewModel)]
#[table_name = "guardians"]
#[model(Guardian)]
pub struct NewGuardian<'a> {
    pub telegram_name: &'a str,
    pub telegram_id: i64,
    pub psn_name: &'a str,
}

impl fmt::Display for Guardian {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (t.me/{})", self.psn_name, self.telegram_name)
    }
}

//
// PlannedActivity
//

// class PlannedActivity(id: EntityID<Int>) : IntEntity(id) {
//     var author by Guardian referencedOn PlannedActivities.authorId
//     var activity by Activity referencedOn PlannedActivities.activityId
//     var start by PlannedActivities.start
//     var details by PlannedActivities.details
//     val members by PlannedActivityMember referrersOn PlannedActivityMembers.plannedActivityId
//     fun membersFormatted(joiner: String): String = members.toList().joinToString(joiner) { it.user.formatName() }
//     fun membersFormattedList(): String = membersFormatted(", ")
//     fun membersFormattedColumn(): String = membersFormatted("\n")
//     fun joinPrompt(): String = if (isFull()) {
//             "This activity fireteam is full."
//         } else {
//             val count = activity.maxFireteamSize - members.count()
//             "Enter `${joinLink()}` to join this group. Up to $count more can join."
//         }
// }

#[derive(Debug, Queryable, Identifiable, AsChangeset, Associations, Model)]
#[belongs_to(Guardian, foreign_key = "author_id")]
#[belongs_to(Activity, foreign_key = "activity_id")]
#[table_name = "plannedactivities"]
pub struct PlannedActivity {
    pub id: i32,
    pub author_id: i32,   // refs Guardians
    pub activity_id: i32, // refs Activities
    pub details: Option<String>,
    pub start: NaiveDateTime,
}

impl PlannedActivity {
    // pub fn author() -> Guardian {
    // guardians.find(self.author_id).first().load::<Guardian>();pffft
    // }
    pub fn join_link(&self) -> String {
        format!("/join {}", self.id)
    }

    //     fun isFull(): Boolean = members.count() >= activity.maxFireteamSize
    pub fn is_full(&self) -> bool {
        false
    }

    //     fun requiresMoreMembers(): Boolean = members.count() < activity.minFireteamSize
    pub fn requires_more_members(&self) -> bool {
        false
    }

    pub fn format_details(&self) -> String {
        match self.details {
            None => "".to_string(),
            Some(ref x) => format!("{}\n", x),
        }
    }
}

impl fmt::Display for PlannedActivity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<b>{id}</b> {}", self.format_details(), id = self.id)
    }
}

// }.toList().sortedBy { it.start }.map { act ->
//     "<b>${act.id}</b>: <b>${act.activity.formatName()}</b>\n" +
//         act.detailsFormatted() +
//         act.membersFormattedColumn() + "\n" +
//         "⏰ <b>${formatStartTime(act.start)}</b>\n" +
//         act.joinPrompt() + "\n"
// }.joinToString("\n")

//
// PlannedActivityMember
//

#[derive(Debug, Queryable, Identifiable, AsChangeset, Associations, Model)]
#[belongs_to(Guardian, foreign_key = "user_id")]
#[belongs_to(Activity, foreign_key = "planned_activity_id")]
#[table_name = "plannedactivitymembers"]
pub struct PlannedActivityMember {
    pub id: i32,
    pub planned_activity_id: i32,
    pub user_id: i32,
    pub added: NaiveDateTime,
}

//
// PlannedActivityReminder
//

//     var user by Guardian referencedOn PlannedActivityReminders.userId
//     var activity by PlannedActivity referencedOn PlannedActivityReminders.plannedActivityId
//     var reminder by PlannedActivityReminders.remind

#[derive(Debug, Queryable, Identifiable, AsChangeset, Associations, Model)]
#[belongs_to(Guardian, foreign_key = "user_id")]
#[belongs_to(Activity, foreign_key = "planned_activity_id")]
#[table_name = "plannedactivityreminders"]
pub struct PlannedActivityReminder {
    pub id: i32,
    pub planned_activity_id: i32, // refs planned_activities
    pub user_id: i32,             // refs Guardians
    pub remind: NaiveDateTime,
}
