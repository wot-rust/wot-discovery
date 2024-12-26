//! Web of Things Discovery
//!
//! Discover [Web Of Things](https://www.w3.org/WoT/) that advertise themselves in the network.
//!
//! ## Supported Introduction Mechanisms
//!
//! - [x] [mDNS-SD (HTTP)](https://www.w3.org/TR/wot-discovery/#introduction-dns-sd-sec)

use std::{marker::PhantomData, net::IpAddr};

use futures_core::Stream;
use futures_util::StreamExt;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use reqwest::Client;
use tracing::debug;

use wot_td::{
    extend::{Extend, ExtendablePiece, ExtendableThing},
    hlist::Nil,
    thing::Thing,
};

/// The error type for Discovery operation
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("mdns cannot be accessed {0}")]
    Mdns(#[from] mdns_sd::Error),
    #[error("reqwest error {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Missing address")]
    NoAddress,
}

/// A specialized [`Result`] type
pub type Result<T> = std::result::Result<T, Error>;

const WELL_KNOWN: &str = "/.well-known/wot";

/// Discover [Web Of Things](https://www.w3.org/WoT/) via a supported Introduction Mechanism.
pub struct Discoverer<Other: ExtendableThing + ExtendablePiece = Nil> {
    mdns: ServiceDaemon,
    service_type: String,
    _other: PhantomData<Other>,
    http_client: reqwest::Client,
}

/// Discovered Thing and its mDNS information
pub struct Discovered<Other: ExtendableThing + ExtendablePiece> {
    /// Discovered Thing
    ///
    /// It is provided as presented by the discovered Servient.
    pub thing: Thing<Other>,
    info: ServiceInfo,
    scheme: String,
}

impl<Other: ExtendableThing + ExtendablePiece> Discovered<Other> {
    /// Discovered Servient listening addresses
    pub fn get_addresses(&self) -> Vec<IpAddr> {
        self.info
            .get_addresses()
            .iter()
            .map(|ip| ip.to_owned().into())
            .collect()
    }
    /// Discovered Servient listening port
    pub fn get_port(&self) -> u16 {
        self.info.get_port()
    }

    /// Discovered Servient hostname
    ///
    /// To be used to make tls requests
    pub fn get_hostname(&self) -> &str {
        self.info.get_hostname()
    }

    /// Discovered Servient scheme
    pub fn get_scheme(&self) -> &str {
        &self.scheme
    }
}

impl Discoverer {
    /// Creates a new Discoverer
    pub fn new() -> Result<Self> {
        let mdns = ServiceDaemon::new()?;
        let service_type = "_wot._tcp.local.".to_owned();
        let http_client = Client::builder().build()?;
        Ok(Self {
            mdns,
            service_type,
            http_client,
            _other: PhantomData,
        })
    }
}

async fn get_thing<Other: ExtendableThing + ExtendablePiece>(
    client: &Client,
    info: ServiceInfo,
) -> Result<Discovered<Other>> {
    let host = info.get_addresses().iter().next().ok_or(Error::NoAddress)?;
    let port = info.get_port();
    let props = info.get_properties();
    let path = props.get_property_val_str("td").unwrap_or(WELL_KNOWN);
    let proto = props
        .get_property_val_str("scheme")
        .or_else(|| {
            // compatibility with
            props
                .get_property_val_str("tls")
                .map(|tls| if tls == "1" { "https" } else { "http" })
        })
        .unwrap_or("http");

    debug!("Got {proto} {host} {port} {path}");

    let r = client
        .get(format!("{proto}://{host}:{port}{path}"))
        .send()
        .await?;

    let thing = r.json().await?;
    let scheme = proto.to_owned();
    let d = Discovered {
        thing,
        info,
        scheme,
    };

    Ok(d)
}

impl<Other: ExtendableThing + ExtendablePiece> Discoverer<Other> {
    /// Extend the [Discoverer] with a [ExtendableThing]
    pub fn ext<T>(self) -> Discoverer<Other::Target>
    where
        Other: Extend<T>,
        Other::Target: ExtendableThing + ExtendablePiece,
    {
        let Discoverer {
            mdns,
            service_type,
            http_client,
            _other,
        } = self;

        Discoverer {
            mdns,
            service_type,
            http_client,
            _other: PhantomData,
        }
    }

    /// Returns an Stream of discovered things
    pub fn stream(&self) -> Result<impl Stream<Item = Result<Discovered<Other>>>> {
        let receiver = self.mdns.browse(&self.service_type)?;

        let http_client = self.http_client.clone();

        let s = receiver.into_stream().filter_map(move |v| {
            let client = http_client.clone();
            async move {
                tracing::info!("{:?}", v);
                if let ServiceEvent::ServiceResolved(info) = v {
                    let t = get_thing(&client, info).await;
                    Some(t)
                } else {
                    None
                }
            }
        });

        Ok(s)
    }
}
