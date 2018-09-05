ALTER TABLE alerts ALTER COLUMN startdate TYPE timestamp USING startdate AT TIME ZONE 'Europe/Moscow';
ALTER TABLE alerts ALTER COLUMN expirydate TYPE timestamp USING expirydate AT TIME ZONE 'Europe/Moscow';
ALTER TABLE guardians ALTER COLUMN created_at TYPE timestamp USING created_at AT TIME ZONE 'Europe/Moscow';
ALTER TABLE guardians ALTER COLUMN updated_at TYPE timestamp USING updated_at AT TIME ZONE 'Europe/Moscow';
ALTER TABLE guardians ALTER COLUMN deleted_at TYPE timestamp USING deleted_at AT TIME ZONE 'Europe/Moscow';
ALTER TABLE plannedactivities ALTER COLUMN start TYPE timestamp USING start AT TIME ZONE 'Europe/Moscow';
ALTER TABLE plannedactivitymembers ALTER COLUMN added TYPE timestamp USING added AT TIME ZONE 'Europe/Moscow';
