ALTER TABLE alerts ALTER COLUMN startdate TYPE timestamp with time zone USING startdate AT TIME ZONE 'Europe/Moscow';
ALTER TABLE alerts ALTER COLUMN expirydate TYPE timestamp with time zone USING expirydate AT TIME ZONE 'Europe/Moscow';
ALTER TABLE guardians ALTER COLUMN created_at TYPE timestamp with time zone USING created_at AT TIME ZONE 'Europe/Moscow';
ALTER TABLE guardians ALTER COLUMN updated_at TYPE timestamp with time zone USING updated_at AT TIME ZONE 'Europe/Moscow';
ALTER TABLE guardians ALTER COLUMN deleted_at TYPE timestamp with time zone USING deleted_at AT TIME ZONE 'Europe/Moscow';
ALTER TABLE plannedactivities ALTER COLUMN start TYPE timestamp with time zone USING start AT TIME ZONE 'Europe/Moscow';
ALTER TABLE plannedactivitymembers ALTER COLUMN added TYPE timestamp with time zone USING added AT TIME ZONE 'Europe/Moscow';
