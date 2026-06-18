-- M3 Task 1: seed rtc_enabled=false instance row into governance_messaging_config.
-- This is a DATA seed, not a schema change — no CREATE/ALTER.
-- valid_from pinned to a STABLE literal so reruns hit the same row under the
-- unique index (scope,key,valid_from) and ON CONFLICT DO NOTHING is a true no-op.
INSERT INTO governance_messaging_config (scope, key, value_type, value_int, value_bool, value_text, valid_from) VALUES
    ('instance', 'rtc_enabled', 'bool', NULL, false, NULL, '2026-06-18T00:00:00Z'::timestamptz)
ON CONFLICT (scope, key, valid_from) DO NOTHING;
