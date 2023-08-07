use std::io;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::thread::spawn;
use std::fs::File;
use std::path::PathBuf;

use log::{debug, info};
use log4rs;
use daemonize::Daemonize;
use clap::{Parser, Subcommand, Arg};

use crate::config::{
    MainConfig, SerdeJsonMainConfigLoader, MainConfigLoader, UpstreamServerConfig, ServerConfig,
};
use crate::errors::UpstreamNotFoundError;
use crate::ioringbuffer::Ioribnbuffer;

#[derive(Clone, Debug)]
pub struct Upstream {
    pub socket: SocketAddr,
}

impl Upstream {
    pub fn from_config(ucfg: &UpstreamServerConfig) -> Self {
        Self {
            socket: ucfg.socket,
        }
    }
}

pub struct Server {
    pub socket: SocketAddr,
}

impl Server {
    pub fn from_config(scfg: &ServerConfig) -> Self {
        Self {
            socket: scfg.socket,
        }
    }
}

pub trait Balancer {
    fn next_upstream(&mut self) -> Result<&Upstream, UpstreamNotFoundError>;
}

pub struct RoundRobinBalancer {
    upstreams: Vec<Upstream>,
    next_indx: usize,
}

impl RoundRobinBalancer {
    pub fn new() -> Self {
        Self {
            upstreams: vec![],
            next_indx: 0,
        }
    }

    pub fn from_config(cfg: &MainConfig) -> Self {
        let mut upstreams = vec![];
        for ucfg in &cfg.upstream.servers {
            let upstream = Upstream::from_config(&ucfg);
            upstreams.push(upstream);
        }
        Self {
            upstreams: upstreams,
            next_indx: 0,
        }
    }
}

impl Balancer for RoundRobinBalancer {
    fn next_upstream(&mut self) -> Result<&Upstream, UpstreamNotFoundError> {
        self.next_indx = (self.next_indx + 1) % self.upstreams.len();
        return Ok(&self.upstreams[self.next_indx]);
    }
}

fn conn_handler(mut upstream: TcpStream, mut downstream: TcpStream) {
    match upstream.set_nonblocking(true) {
        Ok(_) => {}
        Err(_) => {}
    };
    match downstream.set_nonblocking(true) {
        Ok(_) => {}
        Err(_) => {}
    }

    let upstream_alive = true;
    let downstream_alive = true;
    let mut toupstream_buff = Ioribnbuffer::<4096>::new();
    let mut todownstream_buff = Ioribnbuffer::<4096>::new();

    loop {
        match toupstream_buff.write(&mut downstream) {
            Ok(_) => {}
            Err(ref err) => match err.kind() {
                io::ErrorKind::WouldBlock => {}
                _ => downstream
                    .shutdown(std::net::Shutdown::Both)
                    .expect("shutdown call failed"),
            },
        }

        match toupstream_buff.read(&mut upstream) {
            Ok(_) => {}
            Err(ref err) => match err.kind() {
                io::ErrorKind::WouldBlock => {}
                _ => upstream
                    .shutdown(std::net::Shutdown::Both)
                    .expect("shutdown call failed"),
            },
        }

        match todownstream_buff.write(&mut upstream) {
            Ok(_) => {}
            Err(ref err) => match err.kind() {
                io::ErrorKind::WouldBlock => {}
                _ => upstream
                    .shutdown(std::net::Shutdown::Both)
                    .expect("shutdown call failed"),
            },
        }

        match todownstream_buff.read(&mut downstream) {
            Ok(_) => {}
            Err(ref err) => match err.kind() {
                io::ErrorKind::WouldBlock => {}
                _ => downstream
                    .shutdown(std::net::Shutdown::Both)
                    .expect("shutdown call failed"),
            },
        }
    }
}

pub fn spawn_thread(balancer: &mut dyn Balancer, downstream: TcpStream) {
    if let Ok(upstream) = balancer.next_upstream() {
        let cp = (*upstream).clone();
        let _ = spawn(move || {
            let upstream_conn = TcpStream::connect(cp.socket).unwrap();
            conn_handler(upstream_conn, downstream)
        });
    }
}
