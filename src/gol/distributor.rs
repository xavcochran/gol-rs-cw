use crate::decode::coordinates::Coordinates;
use crate::decode::packet::{DecodeError};
use crate::gol::event::{Event, State};
use crate::gol::io::IoCommand;
use crate::gol::Params;
use crate::util::cell::CellValue;
use anyhow::Result;
use flume::{Receiver, Sender};
use indexmap::IndexSet;
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

    let (coordinate_length, _) = Coordinates::calc_coord_len_and_offset(params.image_height as u32);
    let mut world = preprocess(io_input, params, coordinate_length).unwrap();
    // TODO: Execute all turns of the Game of Life.

    let (result_chan_tx, result_chan_rx) = flume::unbounded::<IndexSet<u32>>();

    // Call broker here


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

/// Creates an indexed set of alive coordinates by combining the x and y value into a single value
///
/// e.g. for a 512x512 world:
/// - `0000 0000 0000 00xx xxxx xxxy yyyy yyyy`
///
/// where there are 14 offset bits, 9 x bits and 9 y bits
///
fn preprocess(io_chan: Receiver<CellValue>, params: Params, coordinate_length: u32) -> Result<Vec<u32>, DecodeError> {
    if params.image_height != params.image_width {
        return Err(DecodeError::Other(format!(
            "Image is not square! Height and width values do not match"
        )));
    }

    // initialise the world which is (width*bytes)x(height*bytes) in capacity
    let mut world: Vec<u32> = Vec::with_capacity(params.image_height * params.image_width);

    // gets coordinate length based of image size
    

    let indiv_coord_len = coordinate_length / 2;

    for x in 0..params.image_width as u32 {
        for y in 0..params.image_height as u32 {
            let cell = io_chan.recv();
            match cell {
                Ok(CellValue::Alive) => {
                    world.push(x << indiv_coord_len as u32 | y);
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

fn write_image(
    io_command_chan: Sender<IoCommand>,
    io_filename_chan: Sender<String>,
    io_output_chan: Sender<CellValue>,
    io_idle_chan: Receiver<bool>,
    io_events_chan: Sender<Event>,
    p: Params,
    coordinate_length: u32,
    alive_cells: IndexSet<u32>,
    turn: u32,
) {
    let filename = format!("{}x{}x{}", p.image_width, p.image_height, turn);
    io_command_chan.send(IoCommand::IoOutput).unwrap();

    // clone because filename used again
    io_filename_chan.send(filename.clone()).unwrap();

    
    let indiv_coord_len = coordinate_length / 2;
    for x in 0..p.image_width as u32 {
        for y in 0..p.image_height as u32 {
            // checks to see if the set of alive cells contains that coordinate
            match alive_cells.contains::<u32>(&(x << indiv_coord_len as u32 | y)) {
                true => {
                    io_output_chan.send(CellValue::Alive).unwrap();
                }
                false => {
                    io_output_chan.send(CellValue::Dead).unwrap();
                }
            }
        }
    }

    io_command_chan.send(IoCommand::IoCheckIdle).unwrap();
    io_idle_chan.recv().unwrap();
    
    io_events_chan.send(Event::ImageOutputComplete { completed_turns: turn, filename }).unwrap();
}
