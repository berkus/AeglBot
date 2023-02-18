use surrealdb::engine::remote::ws::Client;
/// Convert from PostgreSQL/Diesel format to SurrealDB
use {
    diesel::pg::PgConnection,
    diesel::prelude::*,
    diesel_logger::LoggingConnection,
    serde::{Deserialize, Serialize},
};

use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};

pub type DbConnection = LoggingConnection<PgConnection>;

#[tokio::main]
async fn main() {
    // Postgres
    let postgres: DbConnection = DbConnection::new(PgConnection::establish(
        &env::var("POSTGRES_DATABASE_URL").expect("POSTGRES_DATABASE_URL must be set"),
    ));

    // Surreal (localhost:8000)
    let surreal = Surreal::new::<Ws>(
        &env::var("SURREAL_DATABASE_URL").expect("SURREAL_DATABASE_URL must be set"),
    )
    .await?;
    surreal
        .signin(Root {
            username: "aeglbot",
            password: "aeglbot62374527345273",
        })
        .await?;
    surreal.use_ns("aegl").use_db("aeglbot").await?;

    // Do NOT copy Warframe events
    // Copy guardians
    let guardians: Vec<Guardian> =
        Guardian::find_all(connection).expect("Failed to load Guardians");
    // select all guardians, iterate, upsert to surreal
    // Copy Destiny activities catalog
    // Copy planned activities and fireteams
}

/* POSTGRES SCHEMA:

table! {
    activities (id) {
        id -> Int4,
        name -> Text,
        mode -> Nullable<Text>,
        min_fireteam_size -> Int4,
        max_fireteam_size -> Int4,
        min_light -> Nullable<Int4>,
        min_level -> Nullable<Int4>,
    }
}

table! {
    activityshortcuts (id) {
        id -> Int4,
        name -> Text,
        game -> Text,
        link -> Int4, -> activities.id
    }
}

table! {
    guardians (id) {
        id -> Int4,
        telegram_name -> Text,
        telegram_id -> Int8,
        psn_name -> Text,
        email -> Nullable<Text>,
        psn_clan -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        tokens -> Nullable<Jsonb>,
        pending_activation_code -> Nullable<Text>,
        is_admin -> Bool,
        is_superadmin -> Bool,
    }
}

table! {
    plannedactivities (id) {
        id -> Int4,
        author_id -> Int4, -> guardians.id,
        activity_id -> Int4, -> activities.id,
        details -> Nullable<Text>,
        start -> Timestamptz,
    }
}

table! {
    plannedactivitymembers (id) {
        id -> Int4,
        planned_activity_id -> Int4, -> plannedactivities.id,
        user_id -> Int4, -> guardians.id,
        added -> Timestamptz,
    }
}

SurrealDB model:

guardian {
    key,
    telegram_name: Text,
    telegram_id: Int8,
    psn_name: Text,
    email: Nullable<Text>,
    psn_clan: Nullable<Text>,
    created_at: Timestamptz,
    updated_at: Timestamptz,
    deleted_at: Nullable<Timestamptz>,
    tokens: Nullable<Jsonb>, // ?
    pending_activation_code: Nullable<Text>, // ?
    is_admin: Bool,
    is_superadmin: Bool,
}

activity {
    key,
    name: Text,
    mode: Optional<Text>,
    min_fireteam_size: Int4,
    max_fireteam_size: Int4,
    min_light: Nullable<Int4>,
    min_level: Nullable<Int4>,
    shortcuts: Set<Text>, // should be indexable/searchable
    game: Text,
}

plannedactivity {
    id: int, // publicly visible
    author -> Guardian
    activity -> Activity
    members -> (member: Guardian, added: Timestamp)
    details: Nullable<Text>,
    start: Timestamptz,
}

*/
