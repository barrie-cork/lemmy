-- Restore the AFTER INSERT trigger shape from
-- migrations/2026-04-20-000000-0000_add_governance_log_notify/up.sql
-- (which subscribers will observe unsigned rows from — accepting that
-- trade-off is the precondition for running down.sql).

DROP TRIGGER IF EXISTS governance_log_notify_trigger ON governance_log;

CREATE OR REPLACE FUNCTION governance_log_notify() RETURNS TRIGGER AS $$
BEGIN
  PERFORM pg_notify(
    'governance_events',
    json_build_object(
      'entry_id', NEW.id,
      'kind', NEW.entry_kind,
      'created_at', NEW.created_at
    )::text
  );
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER governance_log_notify_trigger
  AFTER INSERT ON governance_log
  FOR EACH ROW
  EXECUTE FUNCTION governance_log_notify();
