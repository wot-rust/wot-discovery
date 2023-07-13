use std::time::Duration;

use futures_util::StreamExt;

use serde::{Deserialize, Serialize};
use wot_discovery::Discoverer;
use wot_serve::servient::*;

use tokio::task;
use wot_td::extend::ExtendableThing;

async fn run_servient() {
    let servient = Servient::builder("TestThing")
        .finish_extend()
        .http_bind("127.0.0.1:8080".parse().unwrap())
        .build_servient()
        .unwrap();

    eprintln!("Listening to 127.0.0.1:8080");

    let _ = tokio::time::timeout(Duration::from_secs(30), async {
        servient.serve().await.unwrap()
    })
    .await;
}

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

#[tokio::test(flavor = "multi_thread")]
async fn discoverer() -> Result<(), Box<dyn std::error::Error>> {
    let local = task::LocalSet::new();

    local
        .run_until(async {
            task::spawn_local(async {
                run_servient().await;
            });

            task::spawn_local(async {
                let d = Discoverer::new().unwrap();

                let t = std::pin::pin!(d.stream().unwrap())
                    .next()
                    .await
                    .unwrap()
                    .unwrap();

                assert_eq!("TestThing", t.title);
            });

            task::spawn_local(async {
                let d = Discoverer::new().unwrap().ext::<A>();

                let t = std::pin::pin!(d.stream().unwrap())
                    .next()
                    .await
                    .unwrap()
                    .unwrap();

                assert_eq!("TestThing", t.title);
            });
        })
        .await;

    Ok(())
}
