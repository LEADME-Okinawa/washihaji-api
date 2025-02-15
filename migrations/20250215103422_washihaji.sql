CREATE TABLE rates (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    country VARCHAR(3) NOT NULL UNIQUE,
    rate NUMERIC NOT NULL
);