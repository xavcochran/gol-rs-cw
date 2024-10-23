use flume::{Receiver, Sender};
use indexmap::IndexSet;
use std::net::TcpStream;

struct Job {
    pub world_slice: IndexSet<u32>,
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

struct Broker {}

// queue slices to job channels
// workers consume jobs
fn run_jobs(b: &Broker, client: &TcpStream, jobs: Jobs) {
    loop {
        match jobs.job_chan_rx.recv() {
            Ok(job) => {}
            Err(e) => {}
        }
    }
}

// remove lifetimes here if problems come up
struct WorkerArgs<'a> {
    y1: u32,
    y2: u32,
    turns: u32,
    threads: u32,
    alive_cells: &'a IndexSet<u32>,
}

// remove lifetimes here if problems come up -> currently using to avoid cloning index set of alive cells
fn worker_request<'a>(b: &Broker, client: &TcpStream, args: &'a WorkerArgs,  response_channel: Sender<IndexSet<u32>>) {
    // TODO: create header based on protocol md. I think create packer header struct then encode it
    // then encode payload data
    // then send data to tcp stream
    // await response and set down response channel

    
}

fn calculate_alive_cells(world: &IndexSet<u32>) -> usize {
    world.len()
}
