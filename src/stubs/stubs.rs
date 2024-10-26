use indexmap::IndexSet;

use crate::broker::broker::Jobs;

pub struct Params {
    pub turns: u32,
    pub threads: u32,
    pub image_width: u32,
    pub image_height: u32,
}

pub struct Subscription {
    pub worker_ip_address: String,
    pub function: String,
    pub jobs: Jobs,
}

pub struct StatusReport {
	pub message: String,
}
pub struct BrokerRequest {
    pub params: Params,
    pub world: IndexSet<u32>
}

pub struct BrokerResponse {
    pub world: IndexSet<u32>,
    pub current_turn: u32,
    pub alive_count: u32,
    pub paused: bool
}

pub struct ProcessSliceArgs {
    pub params: Params,
    pub y1: u32,
    pub y2: u32,
    pub alive_cells: IndexSet<u32>,
}

pub struct ProcessSliceResponse {
    pub alive_cells: IndexSet<u32>,
}

pub struct PacketParams {
    pub fn_call_id: u8,
    pub msg_id: u16,
    pub image_size: u16,
}