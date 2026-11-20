-- Schema v4: vaulted status of the relics (WFCD). Reference: docs/05-DATA-SCHEMAS.md.

-- NULL: unknown (WFCD list absent or relic missing from it).
ALTER TABLE relic ADD COLUMN vaulted INTEGER CHECK (vaulted IN (0, 1));
