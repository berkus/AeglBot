#![feature(duration_constructors_lite)]

use {
    culpa::throws,
    sea_orm::{Database, DatabaseConnection, DbErr},
};

pub mod prelude;

pub mod activities;
pub mod activityshortcuts;
pub mod alerts;
pub mod guardians;
pub mod plannedactivities;
pub mod plannedactivitymembers;

// Old diesel schema for reference:
//
// joinable!(activityshortcuts -> activities (link));
// joinable!(plannedactivities -> activities (activity_id));
// joinable!(plannedactivities -> guardians (author_id));
// joinable!(plannedactivitymembers -> guardians (user_id));
// joinable!(plannedactivitymembers -> plannedactivities (planned_activity_id));
//
// allow_tables_to_appear_in_same_query!(
//     activities,
//     activityshortcuts,
//     alerts,
//     guardians,
//     plannedactivities,
//     plannedactivitymembers,
// );

/// Establish a pool of connections with DB.
#[throws(DbErr)]
pub async fn establish_db_connection() -> DatabaseConnection {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Database::connect(database_url).await?
}
