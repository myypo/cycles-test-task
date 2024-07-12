CREATE TABLE "ingredient" (
    ingredient_id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    name VARCHAR(128) COLLATE "case_insensitive" UNIQUE NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

SELECT trigger_updated_at('"ingredient"');

CREATE TABLE "image_ingredient" (
    external_image_id UUID NOT NULL,
    ingredient_id UUID NOT NULL REFERENCES ingredient (
        ingredient_id
    ) ON DELETE CASCADE,
    PRIMARY KEY (external_image_id, ingredient_id)
);


CREATE TABLE "burger_ingredient" (
    burger_id UUID NOT NULL REFERENCES burger (burger_id) ON DELETE CASCADE,
    ingredient_id UUID NOT NULL REFERENCES ingredient (
        ingredient_id
    ) ON DELETE CASCADE,
    PRIMARY KEY (burger_id, ingredient_id)
);
