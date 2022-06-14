ALTER TABLE samples ADD source1 text;
ALTER TABLE samples ADD source2 text;
UPDATE samples SET source1 = '';
UPDATE samples SET source2 = '';
ALTER TABLE samples ALTER COLUMN source1 SET NOT NULL;
ALTER TABLE samples ALTER COLUMN source2 SET NOT NULL;