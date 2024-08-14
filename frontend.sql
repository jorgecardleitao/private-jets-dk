SET s3_endpoint='fra1.digitaloceanspaces.com';

CREATE TABLE "legs" AS (
    SELECT
        "start", "tail_number", "duration", "co2_emissions", "distance", "great_circle_distance"
    FROM
    read_csv_auto("s3://private-jets/leg/v2/all/year=*/data.csv", header = true)
    WHERE "start" >= '2019-01-01'
);

COPY (
    SELECT
        date_trunc('day', "start") AS "date",
        COUNT(DISTINCT("tail_number")) AS "number_of_aircrafts",
        COUNT(*) AS "number_of_legs",
        SUM("duration") AS "time_flown",
        SUM("co2_emissions") AS "co2_emitted",
        SUM("distance") AS "km_flown",
        SUM("great_circle_distance") AS "km_travelled",
    FROM "legs"
    GROUP BY "date"
    ORDER BY "date"
)
TO 'results/by_day.csv' (HEADER, DELIMITER ',');

COPY (
    SELECT
        date_trunc('month', "start") AS "date",
        COUNT(DISTINCT("tail_number")) AS "number_of_aircrafts",
        COUNT(*) AS "number_of_legs",
        SUM("duration") AS "time_flown",
        SUM("co2_emissions") AS "co2_emitted",
        SUM("distance") AS "km_flown",
        SUM("great_circle_distance") AS "km_travelled",
    FROM "legs"
    GROUP BY "date"
    ORDER BY "date"
)
TO 'results/by_month.csv' (HEADER, DELIMITER ',');

COPY (
    SELECT
        date_trunc('year', "start") AS "date",
        COUNT(DISTINCT("tail_number")) AS "number_of_aircrafts",
        COUNT(*) AS "number_of_legs",
        SUM("duration") AS "time_flown",
        SUM("co2_emissions") AS "co2_emitted",
        SUM("distance") AS "km_flown",
        SUM("great_circle_distance") AS "km_travelled",
    FROM "legs"
    GROUP BY "date"
    ORDER BY "date"
)
TO 'results/by_year.csv' (HEADER, DELIMITER ',');

SELECT 'by_day.csv' AS "table"
UNION ALL
SELECT 'by_month.csv' AS "table"
UNION ALL
SELECT 'by_year.csv' AS "table"
