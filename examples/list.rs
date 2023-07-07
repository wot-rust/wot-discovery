use std::future::ready;

use futures_util::StreamExt;
use tracing::{info, warn};
use wot_discovery::Discoverer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let d = Discoverer::new()?;

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
