use crate::decode::function_call::FunctionCall;
use crate::decode::rpc::{BrokerRpc, Rpc, RpcError, SubscriptionRpc};
use crate::stubs::stubs::{
    BrokerRequest, BrokerResponse, ProcessSliceArgs, StatusReport, Subscription,
};
use core::fmt;

use flume::{Receiver, Sender};
use indexmap::IndexSet;
use std::future::Future;
use std::{any::Any, collections::HashMap};
use tokio::{
    self,
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::{
        self,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

pub enum BrokerErr {
    Io(std::io::Error),
    Other(String),
    ConnectionError(String, std::io::Error),
}

impl fmt::Display for BrokerErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrokerErr::Io(e) => write!(f, "IO error: {}", e),
            BrokerErr::Other(msg) => write!(f, "Error: {}", msg),
            BrokerErr::ConnectionError(msg, e) => {
                write!(f, "Error: {}", format!("{} {}", msg, e))
            }
        }
    }
}

pub struct Job {
    pub args: ProcessSliceArgs,
    pub return_chan_rx: Receiver<IndexSet<u32>>,
    pub return_chan_tx: Sender<IndexSet<u32>>,
}

pub struct Jobs {
    pub job_chan_tx: Sender<Job>,
    pub job_chan_rx: Receiver<Job>,
}
impl Jobs {
    pub fn new(threads: usize) -> Self {
        let (job_chan_tx, job_chan_rx) = flume::bounded::<Job>(threads);
        return Self {
            job_chan_tx,
            job_chan_rx,
        };
    }
}

// describes any function that takes a byte array and ouputs a byte array

struct Broker {
    rpc: Rpc,
}

impl Broker {
    fn new() -> Self {
        let mut broker = Self { rpc: Rpc::new() };
        broker.register_all();
        broker
    }

    fn register_all(&mut self) {
        self.rpc
            .register::<SubscriptionRpc, _, _>(FunctionCall::SUBSCRIBE, Self::subscribe);
        self.rpc
            .register::<BrokerRpc, _, _>(FunctionCall::PROCESS_GOL, Self::process_gol);
        self.rpc
            .register::<BrokerRpc, _, _>(FunctionCall::COUNT_ALIVE, Self::count_alive_cells);
        self.rpc
            .register::<BrokerRpc, _, _>(FunctionCall::QUIT, Self::quit);
        self.rpc
            .register::<BrokerRpc, _, _>(FunctionCall::SCREENSHOT, Self::screenshot);
        self.rpc
            .register::<BrokerRpc, _, _>(FunctionCall::PAUSE, Self::pause);
    }

    pub async fn subscribe<'a>(request: Subscription) -> Result<StatusReport, RpcError> {
        // create tcp connection with worker
        let message: String;
        let client = match net::TcpStream::connect("127.0.0.1:8030").await {
            Ok(conn) => {
                message = format!(
                    "Successfully connected {} to broker",
                    request.worker_ip_address
                );
                conn
            }
            Err(e) => {
                message = format!(
                    "Error subscribing to broker on {}: {}",
                    request.worker_ip_address, e
                );
                eprintln!(
                    "Error subscribing to broker on {}: {}",
                    request.worker_ip_address, e
                );
                return Err(RpcError::Io(e));
            }
        };
        // start running runJobs with client connection

        Ok(StatusReport { message: message })
    }

    pub async fn process_gol(request: BrokerRequest) -> Result<BrokerResponse, RpcError> {
        Err(RpcError::Other(format!("Err")))
        // initialise variables
        // if returning from quit start from request params

        // for turn in turns -> calculate next state

        // set response to final values then set everything to 0
    }

    pub async fn count_alive_cells(request: BrokerRequest) -> Result<BrokerResponse, RpcError> {
        Err(RpcError::Other(format!("Err")))
        // lock
        // calc live cells
        // res current turn = current turn
        // res alive count = count
        // unlock
    }

    pub async fn quit(request: BrokerRequest) -> Result<BrokerResponse, RpcError> {
        Err(RpcError::Other(format!("Err")))
        // lock
        // res world = current world
        // res current turn = current turn
        // res paused = false
        // broker quit = true
        // unlock
    }

    pub async fn screenshot(request: BrokerRequest) -> Result<BrokerResponse, RpcError> {
        Err(RpcError::Other(format!("Err")))
        // lock
        // res world = current world
        // res current turn = current turn
        // unlock
    }

    pub async fn pause(request: BrokerRequest) -> Result<BrokerResponse, RpcError> {
        Err(RpcError::Other(format!("Err")))
        // lock
        // broker pause = !broker pause
        // res paused = broker pause
        // unlock
    }
}

// queue slices to job channels
// workers consume jobs
fn run_jobs(b: &Broker, client: &TcpStream, jobs: Jobs) {
    loop {
        match jobs.job_chan_rx.recv() {
            Ok(job) => {
                worker_request(b, client, job.args, job.return_chan_tx);
            }
            Err(e) => {}
        }
    }
}

// remove lifetimes here if problems come up

fn worker_request(
    b: &Broker,
    client: &TcpStream,
    args: ProcessSliceArgs,
    response_channel: Sender<IndexSet<u32>>,
) {
    // TODO: create header based on protocol md. I think create packer header struct then encode it
    // then encode payload data
    // then send data to tcp stream
    // await response and set down response channel
}

fn calculate_alive_cells(world: &IndexSet<u32>) -> usize {
    world.len()
}

pub async fn main() -> Result<(), BrokerErr> {
    let brokerAddress = "127.0.0.1:8030";

    // broker with functions registered
    let broker = Broker::new();

    // create listener
    let ln = match net::TcpListener::bind(brokerAddress).await {
        Ok(ln) => ln,
        Err(e) => {
            return Err(BrokerErr::ConnectionError(
                format!("couldn't create listener on port {}!", brokerAddress),
                e,
            ))
        }
    };
    Ok(())
}
