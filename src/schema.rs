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
        link -> Int4,
    }
}

table! {
    alerts (id) {
        id -> Int4,
        guid -> Text,
        title -> Text,
        #[sql_name = "type"]
        kind -> Text,
        startdate -> Timestamptz,
        expirydate -> Nullable<Timestamptz>,
        faction -> Nullable<Text>,
        flavor -> Nullable<Text>,
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
        author_id -> Int4,
        activity_id -> Int4,
        details -> Nullable<Text>,
        start -> Timestamptz,
    }
}

table! {
    plannedactivitymembers (id) {
        id -> Int4,
        planned_activity_id -> Int4,
        user_id -> Int4,
        added -> Timestamptz,
    }
}

joinable!(activityshortcuts -> activities (link));
joinable!(plannedactivities -> activities (activity_id));
joinable!(plannedactivities -> guardians (author_id));
joinable!(plannedactivitymembers -> guardians (user_id));
joinable!(plannedactivitymembers -> plannedactivities (planned_activity_id));

allow_tables_to_appear_in_same_query!(
    activities,
    activityshortcuts,
    alerts,
    guardians,
    plannedactivities,
    plannedactivitymembers,
);
