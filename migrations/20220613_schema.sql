CREATE TABLE IF NOT EXISTS projects (
	id bytea PRIMARY KEY NOT NULL,
	admin_key bytea NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS samples (
	id bigserial PRIMARY KEY,
	project_id bytea NOT NULL,
	text1 text NOT NULL,
	text2 text NOT NULL
);

CREATE TABLE IF NOT EXISTS ratings (
	id bigserial PRIMARY KEY,
	project_id bytea NOT NULL,
	sample_id bigint NOT NULL,
	ip bytea NOT NULL,
	rating integer NOT NULL
);
CREATE INDEX IF NOT EXISTS project_id_idx ON ratings (project_id);
CREATE INDEX IF NOT EXISTS sample_id_idx ON ratings (sample_id);
CREATE INDEX IF NOT EXISTS ip_idx ON ratings (ip);