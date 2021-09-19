use anyhow::Error;
use prost::Message;
use std::env;
use gtfs_rt::FeedMessage;
use chrono::NaiveDateTime;
use sqlx::postgres::PgPoolOptions;

const POSITIONS_URI: &str = "http://transitdata.cityofmadison.com/Vehicle/VehiclePositions.pb";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let pool = PgPoolOptions::new()
        .connect(&env::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&pool).await?;

    let rclient = reqwest::Client::new();
    
    let positions_pbuf = rclient.get(POSITIONS_URI).send().await?.bytes().await?;
    let feed = FeedMessage::decode(
        positions_pbuf.as_ref() /* because of a version mismatch btwn my old gtfs_rt crate's bytes and the new one that reqwest is using */
    )?;
    // println!("{:#?}", feed);
    for entity in feed.entity {
        let vehicle = entity.vehicle.clone().expect("no vehicle");
        let ts = vehicle.timestamp.expect("no timestamp");
        let pos = vehicle.position.expect("no position");
        
        sqlx::query!("insert into position_messages (ts, messagedata, latitude, longitude) values ($1, $2, $3, $4)",
            NaiveDateTime::from_timestamp(ts as i64, 0),
            serde_json::to_value(entity)?,
            pos.latitude,
            pos.longitude)
            .execute(&pool)
            .await?;
    }

    Ok(())
}
