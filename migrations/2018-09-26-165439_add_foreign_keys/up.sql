-- Links between activities and activityshortcuts
-- Drop all shortcuts if activity is dropped.
alter table activityshortcuts drop constraint if exists activityshortcuts_link_fkey;
alter table activityshortcuts add constraint activityshortcuts_link_fkey
foreign key (link) references activities(id) on delete cascade on update cascade;

-- Links between activities and plannedactivities
-- Disallow activity drop if plannedactivities exist.
alter table plannedactivities drop constraint if exists plannedactivities_activity_id_fkey;
alter table plannedactivities add constraint plannedactivities_activity_id_fkey
foreign key (activity_id) references activities(id) on delete restrict on update cascade;

-- Links between plannedactivitymembers and plannedactivities
-- Drop all members if planned activity is dropped.
alter table plannedactivitymembers drop constraint if exists plannedactivitymembers_planned_activity_id_fkey;
alter table plannedactivitymembers add constraint plannedactivitymembers_planned_activity_id_fkey
foreign key (planned_activity_id) references plannedactivities(id) on delete cascade on update cascade;

-- Drop member if guardian is dropped.
alter table plannedactivitymembers drop constraint if exists plannedactivitymembers_user_id_fkey;
alter table plannedactivitymembers add constraint plannedactivitymembers_user_id_fkey
foreign key (user_id) references guardians(id) on delete cascade on update cascade;
