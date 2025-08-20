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

impl ToTemplate for entity::entities::plannedactivities::Model {}
