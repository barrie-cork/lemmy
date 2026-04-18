-- Emit a Postgres NOTIFY on channel `governance_events` for every row
-- inserted into `governance_log`. Enables out-of-process subscribers (e.g.
-- the V2 messaging bridge per V2/messaging.md §8.4) to observe governance
-- state transitions in near real time without polling.
--
-- Payload kept to 3 fields (entry_id, kind, created_at) so every
-- notification fits Postgres's ~8KB NOTIFY payload cap. Subscribers needing
-- the full row re-query `governance_log` by id.
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
