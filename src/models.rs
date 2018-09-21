use chrono::prelude::*;
use crate::datetime::{format_start_time, reference_date};
use crate::schema::*;
use crate::DbConnection;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use diesel_derives_traits::{Model, NewModel};
use dotenv::dotenv;
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

impl ActivityShortcut {
    pub fn find_one_by_name(
        connection: &DbConnection,
        act_name: &str,
    ) -> diesel::result::QueryResult<Option<Self>> {
        use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
        use schema::activityshortcuts::dsl::*;

        <Self as ::diesel::associations::HasTable>::table()
            .filter(name.eq(act_name))
            .get_result::<Self>(connection)
            .optional()
            .map_err(|e| e)
    }
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
    pub kind: String,
    #[column_name = "startdate"]
    pub start_date: DateTime<Utc>,
    #[column_name = "expirydate"]
    pub expiry_date: Option<DateTime<Utc>>,
    pub faction: Option<String>,
    pub flavor: Option<String>,
}

#[derive(Clone, Insertable, NewModel)]
#[table_name = "alerts"]
#[model(Alert)]
pub struct NewAlert<'a> {
    pub guid: &'a str,
    pub title: &'a str,
    pub kind: Option<&'a str>,
    #[column_name = "startdate"]
    pub start_date: Option<DateTime<Utc>>,
    #[column_name = "expirydate"]
    pub expiry_date: Option<DateTime<Utc>>,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub tokens: Option<Value>,
    pub pending_activation_code: Option<String>,
    pub is_admin: bool,
}

#[derive(Insertable, NewModel)]
#[table_name = "guardians"]
#[model(Guardian)]
pub struct NewGuardian<'a> {
    pub telegram_name: &'a str,
    pub telegram_id: i64,
    pub psn_name: &'a str,
}

impl Guardian {
    pub fn format_name(&self) -> String {
        format!("{} (t.me/{})", self.psn_name, self.telegram_name)
    }
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
    pub start: DateTime<Utc>,
}

#[derive(Insertable, NewModel)]
#[table_name = "plannedactivities"]
#[model(PlannedActivity)]
pub struct NewPlannedActivity {
    pub author_id: i32,   // refs Guardians
    pub activity_id: i32, // refs Activities
    pub start: DateTime<Utc>,
}

impl PlannedActivity {
    pub fn author(&self, connection: &DbConnection) -> Option<Guardian> {
        Guardian::find_one(connection, &self.author_id)
            .expect("Failed to load PlannedActivity author")
    }

    pub fn activity(&self, connection: &DbConnection) -> Activity {
        Activity::find_one(connection, &self.activity_id)
            .expect("Failed to load associated Activity")
            .expect("PlannedActivity without Activity shouldn't exist")
    }

    pub fn members(&self, connection: &DbConnection) -> Vec<PlannedActivityMember> {
        use schema::plannedactivitymembers::dsl::*;
        plannedactivitymembers
            .filter(planned_activity_id.eq(self.id))
            .load::<PlannedActivityMember>(connection)
            .expect("Failed to load PlannedActivity members")
    }

    pub fn members_count(&self, connection: &DbConnection) -> usize {
        //@TODO replace with proper diesel query
        self.members(connection).len()
    }

    pub fn join_link(&self) -> String {
        format!("/join{}", self.id)
    }

    pub fn cancel_link(&self) -> String {
        format!("/cancel{}", self.id)
    }

    pub fn join_prompt(&self, connection: &DbConnection) -> String {
        if self.is_full(connection) {
            "This activity fireteam is full.".into()
        } else {
            let count = self.activity(connection).max_fireteam_size as usize
                - self.members_count(connection);
            format!(
                "Enter `{joinLink}` to join this group. Up to {count} more can join.",
                joinLink = self.join_link(),
                count = count
            )
        }
    }

    pub fn is_full(&self, connection: &DbConnection) -> bool {
        self.members(connection).len() >= self.activity(connection).max_fireteam_size as usize
    }

    pub fn requires_more_members(&self, connection: &DbConnection) -> bool {
        self.members(connection).len() < self.activity(connection).min_fireteam_size as usize
    }

    pub fn format_details(&self) -> String {
        match self.details {
            None => String::new(),
            Some(ref x) => match x.as_ref() {
                "" => String::new(),
                _ => format!("{}\n", x),
            },
        }
    }

    pub fn members_formatted(&self, connection: &DbConnection, joiner: &str) -> String {
        self.members(connection)
            .into_iter()
            .map(|guardian| guardian.format_name(connection))
            .collect::<Vec<String>>()
            .as_slice()
            .join(joiner)
    }

    pub fn members_formatted_list(&self, connection: &DbConnection) -> String {
        self.members_formatted(connection, ", ")
    }

    pub fn members_formatted_column(&self, connection: &DbConnection) -> String {
        self.members_formatted(connection, "\n")
    }

    pub fn find_member(
        &self,
        connection: &DbConnection,
        g: &Guardian,
    ) -> Option<PlannedActivityMember> {
        use schema::plannedactivitymembers::dsl::*;

        plannedactivitymembers
            .filter(user_id.eq(g.id))
            .filter(planned_activity_id.eq(self.id))
            .first::<PlannedActivityMember>(connection)
            .optional()
            .expect("Failed to run SQL")
    }

    pub fn display(&self, connection: &DbConnection, g: Option<&Guardian>) -> String {
        format!(
            "<b>{id}</b>: <b>{name}</b>
{details}{members}
⏰ <b>{time}</b>
{join}{leave}",
            id = self.id,
            name = self.activity(connection).format_name(),
            details = self.format_details(),
            members = self.members_formatted_column(connection),
            time = format_start_time(self.start, reference_date()),
            join = self.join_prompt(connection),
            leave = if !g.is_none() {
                if !self.find_member(connection, g.unwrap()).is_none() {
                    format!("\nEnter `{}` to leave this group.", self.cancel_link())
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        )
    }
}

// @todo when/if PlannedActivity stores all necessary state locally..
// impl fmt::Display for PlannedActivity {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "<b>{id}</b>: <b>{name}</b>\n{details}{members}\n⏰ <b>{time}</b>\n{join}\n",
//             id = self.id,
//             name = self.activity().format_name(),
//             details = self.format_details(),
//             members = self.members_formatted_column(),
//             time = format_start_time(Local.from_local_datetime(&self.start).unwrap()),
//             join = self.join_prompt()
//         )
//     }
// }

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
    pub added: DateTime<Utc>,
}

#[derive(Insertable, NewModel)]
#[table_name = "plannedactivitymembers"]
#[model(PlannedActivityMember)]
pub struct NewPlannedActivityMember {
    pub planned_activity_id: i32,
    pub user_id: i32,
    pub added: DateTime<Utc>,
}

impl PlannedActivityMember {
    pub fn format_name(&self, connection: &DbConnection) -> String {
        Guardian::find_one(connection, &self.user_id)
            .expect("Failed to load associated Guardian")
            .expect("Failed to find associated activity member")
            .format_name()
    }
}

//=================================================================================================
// Tests
//=================================================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bot;
    use diesel::prelude::*;
    use std::env;
    use tokio_core::reactor::Core;

    #[test]
    fn test_guardians() {
        use crate::schema::guardians::dsl::*;

        dotenv().ok();
        let core = Core::new().unwrap();
        let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
        let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
        let bot = Bot::new(&bot_name, core.handle(), &token);

        let results = guardians
            // .filter(published.eq(true))
            .limit(5)
            .load::<Guardian>(&bot.connection())
            .expect("Error loading guardians");

        println!("Displaying {} guardians", results.len());
        for guar in results {
            println!("{}", guar);
        }
    }

    #[test]
    fn test_activities() {
        use crate::schema::activities::dsl::*;

        dotenv().ok();
        let core = Core::new().unwrap();
        let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
        let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
        let bot = Bot::new(&bot_name, core.handle(), &token);

        let results = activities
            .load::<Activity>(&bot.connection())
            .expect("Error loading activities");

        println!("Displaying {} activities", results.len());
        for act in results {
            println!("{}", act.format_name());
        }
    }

    #[test]
    fn test_alerts() {
        use crate::schema::alerts::dsl::*;

        dotenv().ok();
        let core = Core::new().unwrap();
        let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
        let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
        let bot = Bot::new(&bot_name, core.handle(), &token);

        let results = alerts
            .limit(5)
            .load::<Alert>(&bot.connection())
            .expect("Error loading alerts");

        println!("Displaying {} alerts", results.len());
        for alrt in results {
            println!("{}", alrt.title);
        }
    }

    #[test]
    fn test_planned_activities() {
        use crate::schema::guardians::dsl::*;

        dotenv().ok();
        let core = Core::new().unwrap();
        let bot_name = env::var("TELEGRAM_BOT_NAME").expect("TELEGRAM_BOT_NAME must be set");
        let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
        let bot = Bot::new(&bot_name, core.handle(), &token);

        let guar = guardians
            .find(1)
            .first::<Guardian>(&bot.connection())
            .expect("Guardian with id 1 not found");
        let results = PlannedActivity::belonging_to(&guar)
            .load::<PlannedActivity>(&bot.connection())
            .expect("Error loading activities");

        println!("Displaying {} planned activities", results.len());
        for act in results {
            println!("{}", act.display(&bot.connection(), Some(&guar)));
        }
    }
}
