CREATE TABLE IF NOT EXISTS rates (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    country VARCHAR(3) NOT NULL UNIQUE,
    rate NUMERIC NOT NULL
);

INSERT INTO
    rates ("country", "rate")
VALUES
    ('CNY', 7.614958),
    ('EUR', 1.0),
    ('JPY', 159.92482),
    ('KRW', 1513.178853),
    ('USD', 1.049754);