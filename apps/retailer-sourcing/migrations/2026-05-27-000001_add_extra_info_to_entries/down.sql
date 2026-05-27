ALTER TABLE outbox_entries  DROP COLUMN extra_info;
ALTER TABLE event_entries   DROP COLUMN extra_info;
ALTER TABLE command_entries DROP COLUMN extra_info;
ALTER TABLE inbox_entries   DROP COLUMN extra_info;
