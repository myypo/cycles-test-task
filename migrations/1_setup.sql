CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS trigger AS
$$
begin
    NEW.updated_at = now();
    return NEW;
end;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION trigger_updated_at(tablename regclass)
RETURNS void AS
$$
begin
    execute format('CREATE TRIGGER set_updated_at
        BEFORE UPDATE
        ON %s
        FOR EACH ROW
        WHEN (OLD is distinct from NEW)
    EXECUTE FUNCTION set_updated_at();', tablename);
end;
$$ LANGUAGE plpgsql;

CREATE COLLATION case_insensitive (
    PROVIDER = ICU, LOCALE = 'und-u-ks-level2', DETERMINISTIC = false
);
