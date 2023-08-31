use std::future::ready;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{info, trace, warn};
use tracing_subscriber::EnvFilter;
use wot_discovery::{Discovered, Discoverer};
use wot_td::extend::ExtendableThing;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct A {}

impl ExtendableThing for A {
    type InteractionAffordance = ();
    type PropertyAffordance = ();
    type ActionAffordance = ();
    type EventAffordance = ();
    type Form = ();
    type ExpectedResponse = ();
    type DataSchema = ();
    type ObjectSchema = ();
    type ArraySchema = ();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::fmt().with_env_filter(filter).init();

    let d = Discoverer::new()?.ext::<A>();

    d.stream()?
        .for_each(|discovered| {
            match discovered {
                Ok(Discovered { thing: t, .. }) => {
                    info!("found {:?} {:?}", t.title, t.id,);
                    trace!("{}", serde_json::to_string_pretty(&t).unwrap());
                }
                Err(e) => warn!("something went wrong {:?}", e),
            }
            ready(())
        })
        .await;

    Ok(())
}
