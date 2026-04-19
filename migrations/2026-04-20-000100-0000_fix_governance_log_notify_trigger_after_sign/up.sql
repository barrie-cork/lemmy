-- Correctness fix per PR #10 CodeRabbit re-review F14.
--
-- The original trigger (migrations/2026-04-20-000000-0000_add_governance_log_notify)
-- fires AFTER INSERT, but `signature` is populated by a SEPARATE UPDATE
-- from `governance_log::append` in Rust (crates/api/api/src/governance/governance_log.rs:118).
-- Subscribers observing the INSERT would see rows with `signature IS NULL`
-- even though every appended row is signed moments later. This is a
-- correctness gap for the V2 messaging bridge (V2/messaging.md §8.4) —
-- subscribers need the signed artifact.
--
-- Fix: replace the AFTER INSERT trigger with an AFTER UPDATE OF signature
-- trigger that fires when `signature` transitions from NULL → NOT NULL.
-- That is the exact moment the row becomes a subscribable artifact.
--
-- The `governance_log_signature_gate` trigger (see triggers.sql) allows
-- exactly one NULL → non-NULL transition on `signature` per row and
-- rejects any other signature change, so this trigger fires at most
-- once per row. At-most-once delivery semantics are documented in
-- SUBSCRIPTIONS.md (subscribers must use last_seen_id for catch-up).

DROP TRIGGER IF EXISTS governance_log_notify_trigger ON governance_log;

CREATE OR REPLACE FUNCTION governance_log_notify() RETURNS TRIGGER AS $$
BEGIN
  -- Only notify when signature transitions NULL → NOT NULL. The gate trigger
  -- ensures this happens exactly once per row.
  IF OLD.signature IS NULL AND NEW.signature IS NOT NULL THEN
    PERFORM pg_notify(
      'governance_events',
      json_build_object(
        'entry_id', NEW.id,
        'kind', NEW.entry_kind,
        'created_at', NEW.created_at
      )::text
    );
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER governance_log_notify_trigger
  AFTER UPDATE OF signature ON governance_log
  FOR EACH ROW
  EXECUTE FUNCTION governance_log_notify();
