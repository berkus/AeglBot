//=================================================================================================
// DB Models and Tera templates
//=================================================================================================

use {
    crate::{
        datetime::{format_start_time, reference_date},
        render_template,
        schema::*,
        DbConnection,
    },
    chrono::{prelude::*, Duration},
    diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl},
    diesel_derives_traits::Model,
    serde::{Deserialize, Serialize},
    serde_json::Value,
    std::{fmt, sync::LazyLock},
};

//-------------------------------------------------------------------------------------------------
// ActivityShortcut
//-------------------------------------------------------------------------------------------------

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

//-------------------------------------------------------------------------------------------------
// Activity
//-------------------------------------------------------------------------------------------------

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
        format!("{} {}", self.name, self.mode.clone().unwrap_or_default())
    }
}

//-------------------------------------------------------------------------------------------------
// Alert
//-------------------------------------------------------------------------------------------------

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
            LazyLock::new(|| regex::Regex::new(r"^\d+cr ").unwrap());
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

//-------------------------------------------------------------------------------------------------
// Guardian
//-------------------------------------------------------------------------------------------------

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

    pub fn names(&self) -> (String, String) {
        (self.telegram_name.clone(), self.psn_name.clone())
    }
}

impl fmt::Display for Guardian {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (t.me/{})", self.psn_name, self.telegram_name)
    }
}

//-------------------------------------------------------------------------------------------------
// PlannedActivity
//-------------------------------------------------------------------------------------------------

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

// Output information
#[derive(Serialize, Deserialize)]
pub struct PlannedActivityTemplate {
    pub id: i32,
    pub name: String,
    pub details: String,
    pub members: Vec<ActivityMemberTemplate>,
    pub count: usize,
    pub time: String,
    pub fireteam_full: bool,
    pub fireteam_joined: bool,
    pub join_link: String,
    pub leave_link: String,
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
    pub fn to_template(
        &self,
        guardian: Option<&Guardian>,
        connection: &DbConnection,
    ) -> PlannedActivityTemplate {
        let activity = self.activity(connection);

        let count = activity.max_fireteam_size as usize - self.members_count(connection);

        PlannedActivityTemplate {
            id: self.id,
            name: activity.format_name(),
            details: self.format_details(),
            members: self
                .members(connection)
                .into_iter()
                .map(|m| m.to_template(connection))
                .collect(),
            count,
            time: format_start_time(self.start, reference_date()),
            fireteam_full: count == 0,
            join_link: self.join_prompt(connection),
            fireteam_joined: self.find_member(connection, guardian).is_some(),
            leave_link: self.cancel_link(),
        }
    }

    pub fn upcoming_activities(connection: &DbConnection) -> Vec<PlannedActivity> {
        use {
            crate::{datetime::nowtz, schema::plannedactivities::dsl::*},
            diesel::dsl::IntervalDsl,
        };

        plannedactivities
            .filter(start.ge(nowtz() - 60_i32.minutes()))
            .order(start.asc())
            .load::<PlannedActivity>(connection)
            .expect("TEMP failed to load planned activities @FIXME")
    }

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
        self.details.clone().map(|s| s + "\n").unwrap_or_default()
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
        guardian: Option<&Guardian>,
    ) -> Option<PlannedActivityMember> {
        use crate::schema::plannedactivitymembers::dsl::*;

        guardian.and_then(|g| {
            plannedactivitymembers
                .filter(user_id.eq(g.id))
                .filter(planned_activity_id.eq(self.id))
                .first::<PlannedActivityMember>(connection)
                .optional()
                .expect("Failed to run SQL")
        })
    }

    // Makes a telegram Html formatted display.
    pub fn to_string(&self, connection: &DbConnection, g: Option<&Guardian>) -> String {
        let event = self.to_template(g, connection);
        render_template!("list/event", ("event", &event))
            .expect("Failed to render list event template")
    }
}

//-------------------------------------------------------------------------------------------------
// PlannedActivityMember
//-------------------------------------------------------------------------------------------------

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

#[derive(Serialize, Deserialize)]
pub struct ActivityMemberTemplate {
    pub psn_name: String,
    pub telegram_name: String,
    pub icon: String,
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

    pub fn to_template(&self, connection: &DbConnection) -> ActivityMemberTemplate {
        let (telegram_name, psn_name) = Guardian::find_one(connection, &self.user_id)
            .expect("Failed to load associated Guardian")
            .expect("Failed to find associated activity member")
            .names();
        ActivityMemberTemplate {
            psn_name,
            telegram_name,
            icon: self.icon(),
        }
    }

    pub fn icon(&self) -> String {
        static ICON_POOL: LazyLock<Vec<&str>> = LazyLock::new(|| {
            vec![
                "💂🏻",
                "🕵🏼",
                "🧑🏽‍🏭",
                "🧑‍💻",
                "🧑🏼‍🚒",
                "🧑🏾‍🚀",
                "🥷🏾",
                "🥷🏻",
                "🧙🏽",
                "🧝🏼",
                "🧌",
                "🧛🏼",
                "🧟",
            ]
        });
        ICON_POOL[self.user_id.unsigned_abs() as usize % ICON_POOL.len()].into()
    }
}

//=================================================================================================
// Tests
//=================================================================================================

#[cfg(test)]
mod tests {
    use {super::*, crate::establish_db_connection, diesel::prelude::*};

    #[test]
    #[ignore]
    fn test_guardians() -> Result<(), r2d2::Error> {
        use crate::schema::guardians::dsl::*;

        dotenv::dotenv().ok();
        let pool = establish_db_connection();
        let connection = pool.get()?;

        let results = guardians
            // .filter(published.eq(true))
            .limit(5)
            .load::<Guardian>(&connection)
            .expect("Error loading guardians");

        println!("Displaying {} guardians", results.len());
        for guar in results {
            println!("{}", guar);
        }

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_activities() -> Result<(), r2d2::Error> {
        use crate::schema::activities::dsl::*;

        dotenv::dotenv().ok();
        let pool = establish_db_connection();
        let connection = pool.get()?;

        let results = activities
            .load::<Activity>(&connection)
            .expect("Error loading activities");

        println!("Displaying {} activities", results.len());
        for act in results {
            println!("{}", act.format_name());
        }

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_alerts() -> Result<(), r2d2::Error> {
        use crate::schema::alerts::dsl::*;

        dotenv::dotenv().ok();
        let pool = establish_db_connection();
        let connection = pool.get()?;

        let results = alerts
            .limit(5)
            .load::<Alert>(&connection)
            .expect("Error loading alerts");

        println!("Displaying {} alerts", results.len());
        for alrt in results {
            println!("{}", alrt.title);
        }

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_planned_activities() -> Result<(), r2d2::Error> {
        use crate::schema::guardians::dsl::*;

        dotenv::dotenv().ok();
        let pool = establish_db_connection();
        let connection = pool.get()?;

        let guar = guardians
            .find(1)
            .first::<Guardian>(&connection)
            .expect("Guardian with id 1 not found");
        let results = PlannedActivity::belonging_to(&guar)
            .load::<PlannedActivity>(&connection)
            .expect("Error loading activities");

        println!("Displaying {} planned activities", results.len());
        for act in results {
            println!("{}", act.to_string(&connection, Some(&guar)));
        }

        Ok(())
    }
}
