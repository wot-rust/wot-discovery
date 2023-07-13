use std::future::ready;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use wot_discovery::Discoverer;
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
    tracing_subscriber::fmt().init();

    let d = Discoverer::new()?.ext::<A>();

    d.stream()?
        .for_each(|thing| {
            match thing {
                Ok(t) => info!("found {:?} {:?}", t.title, t.id),
                Err(e) => warn!("something went wrong {:?}", e),
            }
            ready(())
        })
        .await;

    Ok(())
}
