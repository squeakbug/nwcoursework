use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::fs::File;
use std::io;

use log::{debug, info};
use log4rs;
use daemonize::Daemonize;
use clap::{Parser, Subcommand, Arg};

use crate::config::MainConfigLoader;

pub mod balancer;
pub mod config;
pub mod errors;
pub mod ioringbuffer;
pub mod nginx_config;

#[derive(Parser)]
#[command(name = "Most Stupid Balancer Ever (MSBE)")]
#[command(author = "Karl Wuber <squeakbug73@outlook.com>")]
#[command(version = "0.0.1")]
#[command(about = "Does stupid things", long_about = None)]
struct Args {
    /// Sets config file for logging (log4rs)
    #[arg(short, long, value_name = "FILE")]
    logconfig: Option<PathBuf>,

    /// Sets main config file
    #[arg(short, long, value_name = "FILE")]
    msbeconfig: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start proxy
    Start {
        #[arg(short, long)]
        list: bool,
    },
    /// Stop proxy
    Stop {
        #[arg(short, long)]
        list: bool,
    },
}

fn main() -> std::io::Result<()> {
    let logconfig_path: &str;
    let msbeconfig_path: &str;
    let cli = Args::parse();
    if let Some(_logconfig_path) = cli.logconfig.as_deref() {
        logconfig_path = _logconfig_path.to_str().unwrap();
    } else {
        logconfig_path = "logging_config.yaml";
    }
    println!("Value for logconfig: {}", logconfig_path);

    if let Some(_msbeconfig_path) = cli.msbeconfig.as_deref() {
        msbeconfig_path = _msbeconfig_path.to_str().unwrap();
    } else {
        msbeconfig_path = "configs/default.json";
    }
    println!("Value for msbeconfig: {}", msbeconfig_path);

    match &cli.command {
        Some(Commands::Start { list }) => {
            println!("Start")
        }
        Some(Commands::Stop { list }) => {
            println!("Stop")
        }
        None => {}
    }

    log4rs::init_file(logconfig_path, Default::default()).unwrap();
    let config_loader = config::SerdeJsonMainConfigLoader::new();
    let config = config_loader
        .load(msbeconfig_path)
        .expect("Bad config loader load operation");
    let mut balancer = balancer::RoundRobinBalancer::from_config(&config);

    let mut servers = vec![];
    for scfg in &config.servers {
        let server = balancer::Server::from_config(&scfg);
        servers.push(server);
    }

    let mut listeners = vec![];
    for server in servers {
        let listener = TcpListener::bind(server.socket)?;
        listener.set_nonblocking(true).unwrap();
        listeners.push(listener);
    }

    /*
    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/test.pid")
        .chown_pid_file(true)
        .working_directory("/tmp")
        .user("nobody")
        .group("daemon")
        .group(2)
        .umask(0o777)
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => println!("Success, daemonized"),
        Err(e) => eprintln!("Error, {}", e),
    }
    */

    info!("Server started");
    loop {
        for listener in &listeners {
            match listener.accept() {
                Ok((stream, _)) => {
                    balancer::spawn_thread(&mut balancer, stream);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => panic!("encountered IO error: {e}"),
            }
        }
    }
}
