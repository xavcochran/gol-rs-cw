use crate::decode::function_call::FunctionCall;
use crate::decode::packet::{DecodeError, Packet, CURRENT_VERSION};
use crate::decode::rpc::{BrokerRpc, Rpc, RpcError, SubscriptionRpc};
use crate::stubs::stubs::{
    BrokerRequest, BrokerResponse, PacketParams, ProcessSliceArgs, StatusReport, Subscription,
};
use crate::worker;
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

struct Worker {
    pub stream: TcpStream,
    pub rx_chan: Receiver<IndexSet<u32>>,
    // pub tx_chan: Sender<Vec<u8>>,
}

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

    pub async fn subscribe(request: Subscription) -> Result<StatusReport, RpcError> {
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

        // send new client down channel, have thread listening on channel to recieve new cleint and begin running jobs

        Ok(StatusReport { message })
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
fn run_jobs(b: &Broker, mut client: Worker, jobs: Jobs) {
    loop {
        // don't think we need to use tokio::select! here as there is only one thread we are wating on
        match jobs.job_chan_rx.recv() {
            Ok(job) => {
                // TODO: create header based on protocol md.
                // then encode payload data
                // then send data to tcp stream
                // await response and set down response channel
                worker_process_slice_request(b, &mut client, job.args, job.return_chan_tx);
            }
            Err(e) => {}
        }
    }
}

async fn worker_process_slice_request(
    b: &Broker,
    client: &mut Worker,
    args: ProcessSliceArgs,
    response_channel: Sender<IndexSet<u32>>,
) {
    let packet = Packet::new();
    let params = PacketParams {
        fn_call_id: FunctionCall::PROCESS_SLICE,
        msg_id: 0, // TODO : implement msg ID => could implement in encode header function
        image_size: args.params.image_width as u16,
    };
    // TODO: implement capacity
    let cells = IndexSet::new();

    match packet
        .write_cells_from_set(&mut client.stream, params, cells)
        .await
    {
        // if result is successful then wait on channel some how for response
        // //
        // each worker has corresponding message channel.
        // when reader thread recieves message, it sends message to corresponding thread (using hash map)
        Ok(_) => {}
        Err(e) => {}
    };
}

fn calculate_alive_cells(world: &IndexSet<u32>) -> usize {
    world.len()
}

#[tokio::main]
pub async fn main() -> Result<(), BrokerErr> {
    let broker_address = "127.0.0.1:8030";

    // broker with functions registered
    let broker = Broker::new();

    // creates jobs struct for up to 16 workers
    let jobs = Jobs::new(16);

    // create listener
    let ln = match net::TcpListener::bind(broker_address).await {
        Ok(ln) => ln,
        Err(e) => {
            return Err(BrokerErr::ConnectionError(
                format!("couldn't create listener on port {}!", broker_address),
                e,
            ))
        }
    };

    // spawn thread to begin listening for new clients that are published via subscribe

    // listens for connections and gets them waiting on new jobs once successfully connected
    match ln.accept().await {
        Ok((mut client, socket_address)) => {
            println!(
                "Successfully accepted connection on port {}",
                socket_address
            );

            let mut packet = Packet::new();
            // read tcp stream and handle connection
            // read header then handle function call
            let header = match packet.read_header(&mut client).await {
                Ok(header) => header,
                Err(e) => {
                    return Err(BrokerErr::Other(format!("Error decoding header")));
                }
            };
            // add request to queue
            
        }
        Err(e) => {
            return Err(BrokerErr::ConnectionError(
                format!("couldn't accept listener on port {}!", broker_address),
                e,
            ))
        }
    };

    Ok(())
}
