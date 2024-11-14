SET s3_endpoint='fra1.digitaloceanspaces.com';

CREATE TABLE "countries" AS (
    SELECT * FROM 'src/country.json'
);

CREATE TABLE "legs" AS (
    SELECT
        "start", "icao_number", "tail_number", "aircraft_model" AS "model", "duration", "co2_emissions", "distance", "great_circle_distance"
    FROM
    read_csv_auto("s3://private-jets/leg/v2/all/year=*/data.csv", header = true)
    WHERE "start" >= '2019-01-01'
);

CREATE TABLE "legs_with_country" AS (
    SELECT
        "legs".*,
        "countries"."country"
    FROM "legs", "countries"
    WHERE
        ('0x' || "legs"."icao_number")::int64 >= "countries"."start"
        AND ('0x' || "legs"."icao_number")::int64 <= "countries"."end"
);

COPY (
    (SELECT
        "country",
        date_trunc('day', "start") AS "date",
        COUNT(DISTINCT("tail_number")) AS "number_of_aircrafts",
        COUNT(*) AS "number_of_legs",
        SUM("duration") AS "time_flown",
        SUM("co2_emissions") AS "co2_emitted",
        SUM("distance") AS "km_flown",
        SUM("great_circle_distance") AS "km_travelled",
    FROM "legs_with_country"
    GROUP BY "country", "date"
    ORDER BY "country", "date")
    UNION ALL
    (SELECT
        'World' AS "country",
        date_trunc('day', "start") AS "date",
        COUNT(DISTINCT("tail_number")) AS "number_of_aircrafts",
        COUNT(*) AS "number_of_legs",
        SUM("duration") AS "time_flown",
        SUM("co2_emissions") AS "co2_emitted",
        SUM("distance") AS "km_flown",
        SUM("great_circle_distance") AS "km_travelled",
    FROM "legs"
    GROUP BY "date"
    ORDER BY "date")
)
TO 'results/by_country_day.csv' (HEADER, DELIMITER ',');

COPY (
    (SELECT
        "country",
        date_trunc('month', "start") AS "date",
        COUNT(DISTINCT("tail_number")) AS "number_of_aircrafts",
        COUNT(*) AS "number_of_legs",
        SUM("duration") AS "time_flown",
        SUM("co2_emissions") AS "co2_emitted",
        SUM("distance") AS "km_flown",
        SUM("great_circle_distance") AS "km_travelled",
    FROM "legs_with_country"
    GROUP BY "country", "date"
    ORDER BY "country", "date")
    UNION ALL
    (SELECT
        'World' AS "country",
        date_trunc('month', "start") AS "date",
        COUNT(DISTINCT("tail_number")) AS "number_of_aircrafts",
        COUNT(*) AS "number_of_legs",
        SUM("duration") AS "time_flown",
        SUM("co2_emissions") AS "co2_emitted",
        SUM("distance") AS "km_flown",
        SUM("great_circle_distance") AS "km_travelled",
    FROM "legs"
    GROUP BY "date"
    ORDER BY "date")
)
TO 'results/by_country_month.csv' (HEADER, DELIMITER ',');

COPY (
    (SELECT
        "country",
        date_trunc('year', "start") AS "date",
        COUNT(DISTINCT("tail_number")) AS "number_of_aircrafts",
        COUNT(*) AS "number_of_legs",
        SUM("duration") AS "time_flown",
        SUM("co2_emissions") AS "co2_emitted",
        SUM("distance") AS "km_flown",
        SUM("great_circle_distance") AS "km_travelled",
    FROM "legs_with_country"
    GROUP BY "country", "date"
    ORDER BY "country", "date")
    UNION ALL
    (SELECT
        'World' AS "country",
        date_trunc('year', "start") AS "date",
        COUNT(DISTINCT("tail_number")) AS "number_of_aircrafts",
        COUNT(*) AS "number_of_legs",
        SUM("duration") AS "time_flown",
        SUM("co2_emissions") AS "co2_emitted",
        SUM("distance") AS "km_flown",
        SUM("great_circle_distance") AS "km_travelled",
    FROM "legs"
    GROUP BY "date"
    ORDER BY "date")
)
TO 'results/by_country_year.csv' (HEADER, DELIMITER ',');

COPY (
    (SELECT
        "model",
        date_trunc('day', "start") AS "date",
        COUNT(DISTINCT("tail_number")) AS "number_of_aircrafts",
        COUNT(*) AS "number_of_legs",
        SUM("duration") AS "time_flown",
        SUM("co2_emissions") AS "co2_emitted",
        SUM("distance") AS "km_flown",
        SUM("great_circle_distance") AS "km_travelled",
    FROM "legs"
    GROUP BY "model", "date"
    ORDER BY "model", "date")
    UNION ALL
    (SELECT
        'World' AS "model",
        date_trunc('day', "start") AS "date",
        COUNT(DISTINCT("tail_number")) AS "number_of_aircrafts",
        COUNT(*) AS "number_of_legs",
        SUM("duration") AS "time_flown",
        SUM("co2_emissions") AS "co2_emitted",
        SUM("distance") AS "km_flown",
        SUM("great_circle_distance") AS "km_travelled",
    FROM "legs"
    GROUP BY "date"
    ORDER BY "date")
)
TO 'results/by_model_day.csv' (HEADER, DELIMITER ',');

COPY (
    (SELECT
        "model",
        date_trunc('month', "start") AS "date",
        COUNT(DISTINCT("tail_number")) AS "number_of_aircrafts",
        COUNT(*) AS "number_of_legs",
        SUM("duration") AS "time_flown",
        SUM("co2_emissions") AS "co2_emitted",
        SUM("distance") AS "km_flown",
        SUM("great_circle_distance") AS "km_travelled",
    FROM "legs"
    GROUP BY "model", "date"
    ORDER BY "model", "date")
    UNION ALL
    (SELECT
        'World' AS "model",
        date_trunc('month', "start") AS "date",
        COUNT(DISTINCT("tail_number")) AS "number_of_aircrafts",
        COUNT(*) AS "number_of_legs",
        SUM("duration") AS "time_flown",
        SUM("co2_emissions") AS "co2_emitted",
        SUM("distance") AS "km_flown",
        SUM("great_circle_distance") AS "km_travelled",
    FROM "legs"
    GROUP BY "date"
    ORDER BY "date")
)
TO 'results/by_model_month.csv' (HEADER, DELIMITER ',');

COPY (
    (SELECT
        "model",
        date_trunc('year', "start") AS "date",
        COUNT(DISTINCT("tail_number")) AS "number_of_aircrafts",
        COUNT(*) AS "number_of_legs",
        SUM("duration") AS "time_flown",
        SUM("co2_emissions") AS "co2_emitted",
        SUM("distance") AS "km_flown",
        SUM("great_circle_distance") AS "km_travelled",
    FROM "legs"
    GROUP BY "model", "date"
    ORDER BY "model", "date")
    UNION ALL
    (SELECT
        'World' AS "model",
        date_trunc('year', "start") AS "date",
        COUNT(DISTINCT("tail_number")) AS "number_of_aircrafts",
        COUNT(*) AS "number_of_legs",
        SUM("duration") AS "time_flown",
        SUM("co2_emissions") AS "co2_emitted",
        SUM("distance") AS "km_flown",
        SUM("great_circle_distance") AS "km_travelled",
    FROM "legs"
    GROUP BY "date"
    ORDER BY "date")
)
TO 'results/by_model_year.csv' (HEADER, DELIMITER ',');

SELECT 'by_country_day.csv' AS "table"
UNION ALL
SELECT 'by_country_month.csv'
UNION ALL
SELECT 'by_country_year.csv'
UNION ALL
SELECT 'by_model_day.csv'
UNION ALL
SELECT 'by_model_month.csv'
UNION ALL
SELECT 'by_model_year.csv'
