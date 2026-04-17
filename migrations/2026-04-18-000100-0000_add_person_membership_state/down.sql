-- Reverse of up.sql — drop column then enum type.
ALTER TABLE person DROP COLUMN IF EXISTS membership_state;
DROP TYPE IF EXISTS membership_state;
