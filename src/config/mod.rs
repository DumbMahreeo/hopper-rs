use crate::server::{
    router::{ConnectedServer, RouterError},
    Client, Router,
};
use config::{ConfigError, File};
use serde::{de::value::I32Deserializer, Deserialize};
use std::{collections::HashMap, net::SocketAddr, sync::Mutex};

#[derive(Deserialize)]
/// Defines the structure of a config file. Extension can be
pub struct ServerConfig {
    /// listening address
    pub listen: SocketAddr,

    // pub routing: Option<RouterConfig>,
    /// routing configuration
    /// required because no other method is currently supported
    pub routing: RouterConfig,
}

impl ServerConfig {
    /// reads configuration from Config.toml
    /// (more file exts can be supported through config's features)
    pub fn new() -> Result<Self, ConfigError> {
        config::Config::builder()
            .add_source(File::with_name("Config"))
            .build()?
            .try_deserialize()
    }
}

struct Balanced {
    servers: Vec<SocketAddr>,
    last_used: usize,
}

impl<'de> Deserialize<'de> for Balanced {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let servers = Vec::deserialize(deserializer)?;
        Ok(Self {
            servers,
            last_used: Default::default(),
        })
    }
}

impl Balanced {
    fn get(&mut self) -> SocketAddr {
        let item = self.servers[self.last_used];
        self.last_used = (self.last_used + 1) % self.servers.len();

        item
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RouteType {
    Simple(SocketAddr),
    Balanced(Mutex<Balanced>),
}

impl RouteType {
    fn get(&self) -> SocketAddr {
        match self {
            RouteType::Simple(route) => *route,
            RouteType::Balanced(balancer) => balancer.lock().unwrap().get(),
        }
    }
}

#[derive(Deserialize)]
pub struct RouterConfig {
    default: Option<SocketAddr>,
    routes: HashMap<String, RouteType>,
}

#[async_trait::async_trait]
impl Router for RouterConfig {
    async fn route(&self, client: &Client) -> Result<ConnectedServer, RouterError> {
        let destination = client.destination();
        self.routes
            // tries to read from hashmap
            .get(destination)
            .map(|dest| dest.get())
            // if not present, uses the optional default
            .or(self.default)
            //     // in case both return None
            .ok_or(RouterError::NoServer)
            // create a future which connects but does not
            // instanciate a minecraft session
            .map(ConnectedServer::connect)?
            .await

        // self.route
        // todo!()
    }
}
