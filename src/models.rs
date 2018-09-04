use super::commands::format_start_time;
use super::schema::*;
use chrono::prelude::*;
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

#[derive(Insertable, NewModel)]
#[table_name = "plannedactivities"]
#[model(PlannedActivity)]
pub struct NewPlannedActivity {
    pub author_id: i32,   // refs Guardians
    pub activity_id: i32, // refs Activities
    pub start: NaiveDateTime,
}

impl PlannedActivity {
    // pub fn author() -> Guardian {
    //     guardians.find(self.author_id).first().load::<Guardian>();pffft
    // }
    // pub fn activity(&self) -> Activity {
    //     PlannedActivity::belonging_to(self.activity_id)
    // }

    pub fn join_link(&self) -> String {
        format!("/join{}", self.id)
    }

    pub fn join_prompt(&self) -> String {
        if self.is_full() {
            format!("This activity fireteam is full.")
        } else {
            let count = 5; //activity.max_fireteam_size - members.count();
            format!(
                "Enter `{joinLink}` to join this group. Up to {count} more can join.",
                joinLink = self.join_link(),
                count = count
            )
        }
    }

    // fun isFull(): Boolean = members.count() >= activity.maxFireteamSize
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

    pub fn members_formatted(&self, joiner: &str) -> String {
        // members.toList().joinToString(joiner) { it.user.formatName() }
        joiner.to_owned()
    }

    pub fn members_formatted_list(&self) -> String {
        self.members_formatted(", ")
    }

    pub fn members_formatted_column(&self) -> String {
        self.members_formatted("\n")
    }
}

impl fmt::Display for PlannedActivity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<b>{id}</b>: <b>{name}</b>\n{details}{members}\n⏰ <b>{time}</b>\n{join}\n",
            id = self.id,
            name = "test", //self.activity().format_name(),
            details = self.format_details(),
            members = self.members_formatted_column(),
            time = format_start_time(Local.from_local_datetime(&self.start).unwrap()),
            join = self.join_prompt()
        )
    }
}

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

#[derive(Insertable, NewModel)]
#[table_name = "plannedactivitymembers"]
#[model(PlannedActivityMember)]
pub struct NewPlannedActivityMember {
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
