CREATE TABLE "burger" (
    burger_id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    name VARCHAR(128) COLLATE "case_insensitive" UNIQUE NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

SELECT trigger_updated_at('"burger"');

CREATE TABLE "image_burger" (
    external_image_id UUID NOT NULL,
    burger_id UUID NOT NULL REFERENCES burger (burger_id) ON DELETE CASCADE,
    PRIMARY KEY (external_image_id, burger_id)
);

CREATE TABLE "tag" (
    tag_id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    name VARCHAR(128) COLLATE "case_insensitive" UNIQUE NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

SELECT trigger_updated_at('"tag"');

CREATE TABLE "burger_tag" (
    burger_id UUID NOT NULL REFERENCES burger (burger_id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tag (tag_id) ON DELETE CASCADE,
    PRIMARY KEY (burger_id, tag_id)
);
