use crate::decode::packet::{Coordinates, DecodeError};
use crate::gol::event::{Event, State};
use crate::gol::io::IoCommand;
use crate::gol::Params;
use crate::util::cell::CellValue;
use anyhow::Result;
use flume::{Receiver, Sender};
use sdl2::keyboard::Keycode;

const BYTE: usize = 8;
pub struct DistributorChannels {
    pub events: Option<Sender<Event>>,
    pub key_presses: Option<Receiver<Keycode>>,
    pub io_command: Option<Sender<IoCommand>>,
    pub io_idle: Option<Receiver<bool>>,
    pub io_filename: Option<Sender<String>>,
    pub io_input: Option<Receiver<CellValue>>,
    pub io_output: Option<Sender<CellValue>>,
}

pub fn distributor(params: Params, mut channels: DistributorChannels) -> Result<()> {
    let events = channels.events.take().unwrap();
    let key_presses = channels.key_presses.take().unwrap();
    let io_command = channels.io_command.take().unwrap();
    let io_idle = channels.io_idle.take().unwrap();
    let io_input = channels.io_input.take().unwrap();
    // TODO: Create a 2D vector to store the world.

    let turn = 0;
    events.send(Event::StateChange {
        completed_turns: turn,
        new_state: State::Executing,
    })?;


    let mut world = preprocess(io_input, params);
    // TODO: Execute all turns of the Game of Life.

    // TODO: Report the final state using FinalTurnCompleteEvent.

    // Make sure that the Io has finished any output before exiting.
    io_command.send(IoCommand::IoCheckIdle)?;
    io_idle.recv()?;

    events.send(Event::StateChange {
        completed_turns: turn,
        new_state: State::Quitting,
    })?;
    Ok(())
}


/// Creates a vector of alive coordinates by combining the x and y value into a single value
/// 
/// e.g. for a 512x512 world:
/// - `0000 0000 0000 00xx xxxx xxxy yyyy yyyy`
/// 
/// where there are 14 offset bits, 9 x bits and 9 y bits
fn preprocess(io_chan: Receiver<CellValue>, params: Params) -> Result<Vec<u32>, DecodeError> {
    if params.image_height != params.image_width {
        return Err(DecodeError::Other(format!(
            "Image is not square! Height and width values do not match"
        ))); 
    }
    
    // initialise the world which is (width*bytes)x(height*bytes) in capacity
    let mut world: Vec<u32> =
        Vec::with_capacity(params.image_height * params.image_width);
    
    // gets coordinate length based of image size
    let (coordinate_length, _) =
        Coordinates::calc_coord_len_and_offset(params.image_height as u32);

    let indiv_coord_len = coordinate_length/2;

    for x in 0..params.image_width as u32 {
        for y in 0..params.image_height as u32 {
            let cell = io_chan.recv();
            match cell {
                Ok(CellValue::Alive) => {
                    world.push(x << indiv_coord_len as u32 | y)
                }
                Ok(CellValue::Dead) => continue,
                Err(e) => {
                    return Err(DecodeError::Other(format!(
                        "Error recieving from channel: {}",
                        e
                    )));
                }
            }
        }
    }
    return Ok(world);
}
