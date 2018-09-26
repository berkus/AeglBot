alter table activityshortcuts drop constraint if exists activityshortcuts_link_fkey;
alter table activityshortcuts add constraint activityshortcuts_link_fkey
foreign key (link) references activities(id) on delete restrict;

alter table plannedactivities drop constraint if exists plannedactivities_activity_id_fkey;
alter table plannedactivities add constraint plannedactivities_activity_id_fkey
foreign key (activity_id) references activities(id);

alter table plannedactivitymembers drop constraint if exists plannedactivitymembers_planned_activity_id_fkey;
alter table plannedactivitymembers add constraint plannedactivitymembers_planned_activity_id_fkey
foreign key (planned_activity_id) references plannedactivities(id);

alter table plannedactivitymembers drop constraint if exists plannedactivitymembers_user_id_fkey;
alter table plannedactivitymembers add constraint plannedactivitymembers_user_id_fkey
foreign key (user_id) references guardians(id);
