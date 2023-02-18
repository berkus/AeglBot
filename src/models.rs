use {
    crate::{
        datetime::{format_start_time, reference_date},
        schema::*,
        DbConnection,
    },
    chrono::{prelude::*, Duration},
    diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl},
    diesel_derives_traits::Model,
    serde_json::Value,
    std::{fmt, sync::LazyLock},
};

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

#[derive(Clone, Insertable, NewModel)]
#[table_name = "activityshortcuts"]
#[model(ActivityShortcut)]
pub struct NewActivityShortcut {
    pub name: String,
    pub game: String,
    pub link: i32,
}

impl ActivityShortcut {
    pub fn find_one_by_name(
        connection: &DbConnection,
        act_name: &str,
    ) -> diesel::result::QueryResult<Option<Self>> {
        use crate::schema::activityshortcuts::dsl::*;

        <Self as ::diesel::associations::HasTable>::table()
            .filter(name.eq(act_name))
            .get_result::<Self>(connection)
            .optional()
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

#[derive(Clone, Insertable, NewModel)]
#[table_name = "activities"]
#[model(Activity)]
pub struct NewActivity {
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
            self.mode.clone().unwrap_or(String::new())
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
pub struct NewAlert {
    pub guid: String,
    pub title: String,
    pub kind: Option<String>,
    #[column_name = "startdate"]
    pub start_date: Option<DateTime<Utc>>,
    #[column_name = "expirydate"]
    pub expiry_date: Option<DateTime<Utc>>,
    pub faction: Option<String>,
    pub flavor: Option<String>,
}

impl fmt::Display for Alert {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.type_icon(),
            self.reward_icon(),
            self.title
        )
    }
}

impl Alert {
    pub fn is_important(&self) -> bool {
        self.is_forma()
            || self.is_nitain()
            || self.is_orokin_reactor()
            || (self.expiry_date.is_some()
                && self.expiry_date.unwrap() - self.start_date >= Duration::minutes(90))
    }

    pub fn type_icon(&self) -> String {
        match self.kind.as_str() {
            "Alert" => "✊".into(),
            "Invasion" => "🐛".into(),
            "Outbreak" => "⛓".into(),
            _ => format!("⁉️ {}", self.kind),
        }
    }

    pub fn reward_icon(&self) -> String {
        if self.is_forma() {
            "⚖"
        } else if self.is_nitain() {
            "✨"
        } else if self.is_orokin_reactor() {
            "🏮"
        } else if self.is_endo() {
            "🔮"
        } else if self.is_blueprint() {
            "🗿"
        } else if self.is_resource() {
            "🔋"
        } else if self.is_mod() {
            "⚙"
        } else if self.is_aura() {
            "❄️"
        } else if self.is_credits() {
            "💰"
        } else {
            ""
        }
        .into()
    }

    pub fn is_blueprint(&self) -> bool {
        self.title.contains("(Blueprint)")
    }

    pub fn is_resource(&self) -> bool {
        self.title.contains("(Resource)")
    }

    pub fn is_mod(&self) -> bool {
        self.title.contains("(Mod)")
    }

    pub fn is_aura(&self) -> bool {
        self.title.contains("(Aura)")
    }

    pub fn is_credits(&self) -> bool {
        static CREDITS: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r#"^\d+cr "#).unwrap());
        CREDITS.is_match(&self.title)
    }

    pub fn is_forma(&self) -> bool {
        self.title.contains("Forma")
    }

    pub fn is_nitain(&self) -> bool {
        self.title.contains("Nitain Extract")
    }

    pub fn is_orokin_reactor(&self) -> bool {
        self.title.contains("Orokin Reactor")
    }

    pub fn is_endo(&self) -> bool {
        self.title.contains("ENDO")
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
    pub is_superadmin: bool,
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
        use crate::schema::plannedactivitymembers::dsl::*;
        plannedactivitymembers
            .filter(planned_activity_id.eq(self.id))
            .order(added.asc())
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
        self.details
            .clone()
            .map(|s| s + "\n")
            .unwrap_or(String::new())
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
        use crate::schema::plannedactivitymembers::dsl::*;

        plannedactivitymembers
            .filter(user_id.eq(g.id))
            .filter(planned_activity_id.eq(self.id))
            .first::<PlannedActivityMember>(connection)
            .optional()
            .expect("Failed to run SQL")
    }

    // Makes a telegram markdown formatted display.
    pub fn display(&self, connection: &DbConnection, g: Option<&Guardian>) -> String {
        format!(
            "<b>{id}</b>: <b>{name}</b>
{details}{members}
⏰ <b>{time}</b>
{join}{leave}",
            id = self.id,
            name = self.activity(connection).format_name(),
            details = teloxide::utils::html::escape(&self.format_details()),
            members = self.members_formatted_column(connection),
            time = format_start_time(self.start, reference_date()),
            join = self.join_prompt(connection),
            leave = g
                .and_then(|g| self.find_member(connection, g))
                .map(|_| format!("\nEnter `{}` to leave this group.", self.cancel_link()))
                .unwrap_or(String::new())
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

#[cfg(testing)]
mod tests {
    use {super::*, crate::Bot, diesel::prelude::*, std::env};

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
