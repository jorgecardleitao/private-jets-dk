use std::{
    collections::{HashMap, HashSet},
    error::Error,
    sync::Arc,
};

use clap::Parser;
use flights::{BlobStorageProvider, Leg};
use futures::{StreamExt, TryStreamExt};
use serde::{de::DeserializeOwned, Serialize};
use simple_logger::SimpleLogger;

static DATABASE_ROOT: &'static str = "leg/v2/";
static DATABASE: &'static str = "leg/v2/data/";

#[derive(serde::Serialize, serde::Deserialize)]
struct LegOut {
    icao_number: Arc<str>,
    #[serde(with = "time::serde::rfc3339")]
    start: time::OffsetDateTime,
    start_lat: f64,
    start_lon: f64,
    start_altitude: f64,
    #[serde(with = "time::serde::rfc3339")]
    end: time::OffsetDateTime,
    end_lat: f64,
    end_lon: f64,
    end_altitude: f64,
    length: f64,
}

#[derive(serde::Serialize)]
struct Metadata {
    icao_months_to_process: usize,
    icao_months_processed: usize,
    url: String,
}

async fn write_json(
    client: &dyn BlobStorageProvider,
    d: impl Serialize,
    key: &str,
) -> Result<(), Box<dyn Error>> {
    let mut bytes: Vec<u8> = Vec::new();
    serde_json::to_writer(&mut bytes, &d).map_err(std::io::Error::other)?;

    Ok(client.put(key, bytes).await?)
}

async fn write_csv(
    items: impl Iterator<Item = impl Serialize>,
    key: &str,
    client: &dyn BlobStorageProvider,
) -> Result<(), std::io::Error> {
    let data_csv = flights::csv::serialize(items);
    client.put(&key, data_csv).await?;
    Ok(())
}

fn transform<'a>(icao_number: &'a Arc<str>, legs: Vec<Leg>) -> impl Iterator<Item = LegOut> + 'a {
    legs.into_iter().map(|leg| LegOut {
        icao_number: icao_number.clone(),
        start: leg.from().datetime(),
        start_lat: leg.from().latitude(),
        start_lon: leg.from().longitude(),
        start_altitude: leg.from().altitude(),
        end: leg.to().datetime(),
        end_lat: leg.to().latitude(),
        end_lon: leg.to().longitude(),
        end_altitude: leg.to().altitude(),
        length: leg.length(),
    })
}

async fn write(
    icao: &Arc<str>,
    month: time::Date,
    legs: impl Iterator<Item = impl Serialize>,
    client: &dyn BlobStorageProvider,
) -> Result<(), Box<dyn Error>> {
    let key = pk_to_blob_name(icao, month);

    write_csv(legs, &key, client).await?;
    log::info!("Written {} {}", icao, month);
    Ok(())
}

async fn read<D: DeserializeOwned>(
    icao: &Arc<str>,
    month: time::Date,
    client: &dyn BlobStorageProvider,
) -> Result<Vec<D>, std::io::Error> {
    flights::io::get_csv(&pk_to_blob_name(icao, month), client).await
}

fn pk_to_blob_name(icao: &str, month: time::Date) -> String {
    format!(
        "{DATABASE}month={}/icao_number={icao}/data.json",
        flights::serde::month_to_part(month)
    )
}

fn blob_name_to_pk(blob: &str) -> (Arc<str>, time::Date) {
    let keys = flights::serde::hive_to_map(&blob[DATABASE.len()..blob.len() - "data.json".len()]);
    let icao = *keys.get("icao_number").unwrap();
    let date = *keys.get("month").unwrap();
    (icao.into(), flights::serde::parse_month(date))
}

/// Returns the set of (icao number, month) that exist in the container prefixed by `prefix`
async fn list(
    client: &dyn BlobStorageProvider,
) -> Result<HashSet<(Arc<str>, time::Date)>, std::io::Error> {
    Ok(client
        .list(DATABASE)
        .await?
        .into_iter()
        .map(|blob| blob_name_to_pk(&blob))
        .collect())
}

const ABOUT: &'static str = r#"Builds the database of all legs"#;

#[derive(Parser, Debug)]
#[command(author, version, about = ABOUT)]
struct Cli {
    /// The token to the remote storage
    #[arg(long)]
    access_key: String,
    /// The token to the remote storage
    #[arg(long)]
    secret_access_key: String,
    /// Optional country to fetch from (in ISO 3166); defaults to whole world
    #[arg(long)]
    country: Option<String>,
}

async fn etl_task(
    icao_number: &Arc<str>,
    month: time::Date,
    client: &dyn BlobStorageProvider,
) -> Result<(), Box<dyn Error>> {
    // extract
    let positions = flights::get_month_positions(&icao_number, month, client).await?;
    // transform
    let legs = transform(&icao_number, flights::legs(positions.into_iter()));
    // load
    write(&icao_number, month, legs, client).await
}

async fn aggregate(
    required: HashSet<(Arc<str>, time::Date)>,
    client: &dyn BlobStorageProvider,
) -> Result<(), Box<dyn Error>> {
    let completed = list(client)
        .await?
        .into_iter()
        .filter(|key| required.contains(key))
        .collect::<HashSet<_>>();

    // group completed by year
    let completed_by_year =
        completed
            .into_iter()
            .fold(HashMap::<i32, HashSet<_>>::new(), |mut acc, v| {
                acc.entry(v.1.year())
                    .and_modify(|entries| {
                        entries.insert(v.clone());
                    })
                    .or_insert(HashSet::from([v]));
                acc
            });
    let required_by_year =
        required
            .into_iter()
            .fold(HashMap::<i32, HashSet<_>>::new(), |mut acc, v| {
                acc.entry(v.1.year())
                    .and_modify(|entries| {
                        entries.insert(v.clone());
                    })
                    .or_insert(HashSet::from([v]));
                acc
            });

    // run tasks by year
    let mut metadata = HashMap::<i32, Metadata>::new();
    for (year, completed) in completed_by_year {
        let tasks = completed.iter().map(|(icao_number, date)| async move {
            read::<LegOut>(icao_number, *date, client).await
        });

        log::info!("Gettings all legs for year={year}");
        let legs = futures::stream::iter(tasks)
            .buffered(100)
            .try_collect::<Vec<_>>()
            .await?
            .into_iter()
            .flatten();

        log::info!("Writing all legs for year={year}");
        let key = format!("{DATABASE_ROOT}all/year={year}/data.csv");
        write_csv(legs, &key, client).await?;
        log::info!("Written {key}");
        metadata.insert(
            year,
            Metadata {
                icao_months_to_process: required_by_year.get(&year).unwrap().len(),
                icao_months_processed: completed.len(),
                url: format!("https://private-jets.fra1.digitaloceanspaces.com/{key}"),
            },
        );
    }

    let key = format!("{DATABASE_ROOT}status.json");
    write_json(client, metadata, &key).await?;
    log::info!("status written");
    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let cli = Cli::parse();
    let maybe_country = cli.country.as_deref();

    let client = flights::fs_s3::client(cli.access_key, cli.secret_access_key).await;
    let client = &client;

    let required =
        flights::private_jets_in_month((2019..2025).rev(), maybe_country, client).await?;

    log::info!("required : {}", required.len());

    let completed = list(client).await?.into_iter().collect::<HashSet<_>>();
    log::info!("completed: {}", completed.len());

    let ready = flights::list_months_positions(client)
        .await?
        .into_iter()
        .filter(|key| required.contains(key))
        .collect::<HashSet<_>>();
    log::info!("ready    : {}", ready.len());

    let mut todo = ready.difference(&completed).collect::<Vec<_>>();
    todo.sort_unstable_by_key(|(icao, date)| (date, icao));
    log::info!("todo     : {}", todo.len());

    let tasks = todo
        .into_iter()
        .map(|(icao_number, month)| async move { etl_task(icao_number, *month, client).await });

    let _ = futures::stream::iter(tasks)
        .buffered(50)
        .collect::<Vec<_>>()
        .await;

    aggregate(required, client).await
}
