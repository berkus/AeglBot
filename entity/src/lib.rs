mod entities;
pub use entities::*;

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
