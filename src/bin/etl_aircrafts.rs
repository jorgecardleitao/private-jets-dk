use std::collections::HashMap;
use std::error::Error;

use clap::Parser;
use futures::StreamExt;
use futures::TryStreamExt;
use simple_logger::SimpleLogger;
use time::Date;

use flights::aircraft;
use flights::fs;
use flights::fs::BlobStorageProvider;

#[derive(clap::ValueEnum, Debug, Clone)]
enum Backend {
    Disk,
    Remote,
}

async fn write_csv(
    items: impl Iterator<Item = impl serde::Serialize>,
    key: &str,
    client: &dyn BlobStorageProvider,
) -> Result<(), std::io::Error> {
    let data_csv = flights::csv::serialize(items);
    client.put(&key, data_csv).await?;
    Ok(())
}

const ABOUT: &'static str = r#"Creates a new snapshot of the database of all worldwide aircrafts according to `M-aircrafts-in-time`.
This ETL is append only - every time it runs, it creates a new snapshot.
If `access_key` and `secret_access_key` are not provided, data is written to the local disk.
"#;

#[derive(Parser, Debug)]
#[command(author, version, about = ABOUT)]
struct Cli {
    /// The token to the remote storage
    #[arg(long)]
    access_key: Option<String>,
    /// The token to the remote storage
    #[arg(long)]
    secret_access_key: Option<String>,
}

fn pk_to_blob_name(month: time::Date) -> String {
    let month = flights::serde::month_to_part(month);
    format!("private_aircraft/v1/month={month}/data.csv")
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let cli = Cli::parse();

    // initialize client
    let client = match (cli.access_key, cli.secret_access_key) {
        (Some(access_key), Some(secret_access_key)) => {
            Some(flights::fs_s3::client(access_key, secret_access_key).await)
        }
        (None, None) => None,
        _ => {
            return Err("both access_key and secret_access_key must be provided or neither".into())
        }
    };
    let client = client
        .as_ref()
        .map(|x| x as &dyn fs::BlobStorageProvider)
        .unwrap_or(&fs::LocalDisk);

    log::info!("Fetching and writing all aircrafts");
    aircraft::etl_aircrafts(client).await?;
    log::info!("All aircrafts written");

    // write private jets to dedicated place.
    log::info!("Writing all models");
    let data = std::fs::read_to_string("src/models.csv")?;
    client
        .put("model/db/data.csv", data.as_bytes().to_vec())
        .await?;
    log::info!("All models written");

    log::info!("Fetching all private aircrafts in time");
    let tasks = flights::private_jets_in_month(2019..2030, None, client).await?;

    let by_month = tasks
        .into_iter()
        .map(|((_, date), (aircraft, _))| (date, aircraft))
        .fold(
            HashMap::<Date, Vec<_>>::new(),
            |mut acc, (date, aircraft)| {
                acc.entry(date)
                    .and_modify(|entries| {
                        entries.push(aircraft.clone());
                    })
                    .or_insert(vec![aircraft]);
                acc
            },
        );

    let tasks = by_month.into_iter().map(|(date, aircrafts)| async move {
        write_csv(aircrafts.into_iter(), &pk_to_blob_name(date), client).await
    });

    futures::stream::iter(tasks)
        .buffered(400)
        .try_collect::<Vec<_>>()
        .await?;

    log::info!("All private aircrafts in time written");

    Ok(())
}
