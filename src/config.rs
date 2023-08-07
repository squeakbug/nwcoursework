use std::{net::SocketAddr, vec, str::FromStr, fs::File, io::prelude::*};

use serde_json;

use crate::{nginx_config, errors::ConfigParseError};

pub enum BalancingPolicy {
    RR,
    Random,
    LeastConnection,
}

pub struct UpstreamServerConfig {
    pub socket: SocketAddr,
}

pub struct UpstreamConfig {
    pub servers: Vec<UpstreamServerConfig>,
    pub policy: BalancingPolicy,
}

pub struct ServerConfig {
    pub socket: SocketAddr,
}

pub struct MainConfig {
    pub servers: Vec<ServerConfig>,
    pub upstream: UpstreamConfig,
}

pub trait MainConfigLoader {
    fn load(&self, path: &str) -> Result<MainConfig, ConfigParseError>;
    fn loads(&self, str_cfg: &str) -> Result<MainConfig, ConfigParseError>;
}

pub struct SerdeJsonMainConfigLoader;

impl SerdeJsonMainConfigLoader {
    pub fn new() -> Self {
        Self {}
    }
}

pub fn cvt_serdejson_to_mainconf(
    serderepr: nginx_config::Root,
) -> Result<MainConfig, ConfigParseError> {
    let http_block = &serderepr.config[0].parsed[0].block;
    let upstream_block = &http_block[0];
    let mut server_blocks = vec![];
    for x in &http_block[1..] {
        server_blocks.push(x);
    }
    let mut upstream_server_blocks = vec![];
    for x in &upstream_block.block[1..] {
        upstream_server_blocks.push(x);
    }

    let mut servers = vec![];
    for server in server_blocks {
        let socket = SocketAddr::from_str(&server.block[0].args[0]).unwrap();
        let scfg = ServerConfig { socket: socket };
        servers.push(scfg);
    }

    let balancing_policy = match upstream_block.block[0].directive.as_str() {
        "round_robin" => Ok(BalancingPolicy::RR),
        "random" => Ok(BalancingPolicy::Random),
        "least_connection" => Ok(BalancingPolicy::LeastConnection),
        _ => Err(ConfigParseError::new()),
    }?;

    let mut upstream_servers = vec![];
    for server in upstream_server_blocks {
        let socket = SocketAddr::from_str(&server.args[0]).unwrap();
        let uscfg = UpstreamServerConfig { socket: socket };
        upstream_servers.push(uscfg);
    }

    Ok(MainConfig {
        servers: servers,
        upstream: UpstreamConfig {
            servers: upstream_servers,
            policy: balancing_policy,
        },
    })
}

impl MainConfigLoader for SerdeJsonMainConfigLoader {
    fn load(&self, path: &str) -> Result<MainConfig, ConfigParseError> {
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        return self.loads(&contents);
    }

    fn loads(&self, str_cfg: &str) -> Result<MainConfig, ConfigParseError> {
        let nginx_conf: nginx_config::Root = serde_json::from_str(&str_cfg).unwrap();
        return cvt_serdejson_to_mainconf(nginx_conf);
    }
}
