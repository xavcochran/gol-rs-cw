use core::fmt;
use std::cell;

use bytes::BytesMut;
use indexmap::IndexSet;
// use std::fmt::Error;
// use std::time::Instant;
// use std::{cell, fmt};
// use std::{fs::read, vec};
use tokio::{io::AsyncReadExt, net::TcpStream};

use super::{
    coordinates::Coordinates,
    function_call::{self, FunctionCall},
};
// originally used standard hashset but doesnt have order
// index set retains order of insertion
// this increases decode time by about 30-40% but i believe it is a worthy tradeoff

pub const BYTE: usize = 8;
pub const BYTE_F64: f64 = 8.0;
const HEADER_SIZE_BYTES: usize = 11;
const VERSION: usize = 0;
const FUNCTION_CALL: usize = 1;
const MESSAGE_ID: usize = 2;
const IMAGE_SIZE: usize = 4;
const LENGTH: usize = 6;
const CHECKSUM: usize = 9;

#[derive(Debug)]
pub enum DecodeError {
    Io(std::io::Error),
    Other(String),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Io(e) => write!(f, "IO error: {}", e),
            DecodeError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    version: u8,
    fn_call: u8,
    msg_id: u16,
    image_size: u16,
    length: u32,
    checksum: u16,
}

impl Header {
    pub fn new() -> Self {
        Self {
            version: 0,
            fn_call: 0,
            msg_id: 0,
            image_size: 0,
            length: 0,
            checksum: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Packet {
    header: Header,
}

impl Packet {
    fn decode_header(&mut self, data: &[u8]) {
        self.header = Header {
            version: data[VERSION],       // first byte
            fn_call: data[FUNCTION_CALL], // second byte
            msg_id: ((data[MESSAGE_ID] as u16) << BYTE | (data[MESSAGE_ID + 1] as u16)), // 3rd & 4th byte
            image_size: ((data[IMAGE_SIZE] as u16) << BYTE | (data[IMAGE_SIZE + 1] as u16)), // 5th & 6th byte
            // 7th -> 10th byte
            length: || -> u32 {
                let mut buf: u32 = 0;
                for byte in &data[LENGTH..LENGTH + 2] {
                    let mut bitcount = 7;
                    buf |= (*byte as u32) << 31 - bitcount;
                    bitcount += byte;
                }
                return buf;
            }(),
            checksum: ((data[CHECKSUM] as u16) << BYTE | (data[CHECKSUM + 1] as u16)), // 11th & 12th byte
        }
    }

    fn decode_payload(
        &mut self,
        data: &[u8],
        coordinate_length: u32,
        offset: u32,
    ) -> IndexSet<u32> {
        let mut buffer: u32 = 0;
        let mut bit_count = 7;
        let mut cells = IndexSet::with_capacity(self.header.length as usize);
        // could borrow here
        let mask: u32 = self.generate_mask(coordinate_length);
        let limit = self.limit(coordinate_length);
        let coordinate_length_usize: usize = coordinate_length as usize;
        for byte in data {
            buffer |= (*byte as u32) << 31 - bit_count; // adds next byte to the buffer
            bit_count += BYTE;

            // n = coordinate_length
            // while there is no space to shift, process first n bits
            while bit_count >= limit {
                let extracted_value = (buffer & mask) >> offset; // get first n bits then shift to right hand side

                cells.insert(extracted_value);

                buffer <<= coordinate_length; // shift buffer to the right by n bits
                bit_count -= coordinate_length_usize; // decrease bit count to account for bits just extracted
            }
        }
        return cells;
    }

    pub async fn decode(&mut self, mut stream: TcpStream) -> Result<IndexSet<u32>, DecodeError> {
        let mut buf = BytesMut::with_capacity(HEADER_SIZE_BYTES);

        let (coordinate_length, offset) =
            Coordinates::calc_coord_len_and_offset(self.header.image_size as u32);

        match stream.read_buf(&mut buf).await {
            Ok(0) => {
                return Err(DecodeError::Other(format!(
                    "Error Reading from stream, read 0 bytes"
                )));
            }
            Ok(n) => {
                if n != 11 {
                    return Err(DecodeError::Other(format!(
                        "Length missmatch, expected headersize of 10, got {}",
                        n
                    )));
                } else {
                    self.decode_header(&buf);
                    let mut payload_buf = BytesMut::with_capacity(self.header.length as usize);
                    match stream.read_buf(&mut payload_buf).await {
                        Ok(0) => {
                            return Err(DecodeError::Other(format!(
                                "Error Reading from stream, read 0 bytes"
                            )));
                        }
                        Ok(n) => {
                            if n != self.header.length as usize {
                                return Err(DecodeError::Other(format!(
                                    "Length missmatch, expected headersize of 10, got {}",
                                    n
                                )));
                            } else {
                                return Ok(self.decode_payload(&buf, coordinate_length, offset));
                            }
                        }
                        Err(e) => {
                            return Err(DecodeError::Other(format!(
                                "Failed to read payload from port; err = {:?}",
                                e
                            )));
                        }
                    }
                }
            }
            Err(e) => {
                return Err(DecodeError::Other(format!(
                    "Failed to read header from port; err = {:?}",
                    e
                )));
            }
        }
    }

    pub fn encode_payload_from_set(
        &self,
        cells: IndexSet<u32>,
        coordinate_length: usize,
    ) -> Vec<u8> {
        let mut buffer: u32 = 0;
        let mut bit_count: usize = coordinate_length - 1;
        let mask: u32 = 0xFF000000;
        let capacity = cells.len() as f64 * (coordinate_length as f64 / BYTE_F64);
        let mut data = Vec::with_capacity(capacity as usize);
        for cell in cells {
            buffer |= cell << 31 - bit_count;
            bit_count += coordinate_length;
            while bit_count >= 32 {
                let byte = buffer & mask;
                bit_count -= BYTE;
                buffer <<= BYTE;

                data.push((byte >> 24) as u8);
            }
        }
        while buffer != 0 {
            let byte = (buffer & 0xFF000000) >> 24;
            data.push(byte as u8);
            buffer <<= BYTE;
        }
        data
    }

    pub fn encode_payload_from_vec(&self, cells: Vec<u32>, coordinate_length: usize) -> Vec<u8> {
        let mut buffer: u32 = 0;
        let mut bit_count: usize = coordinate_length - 1;
        let mask: u32 = 0xFF000000;
        let capacity = cells.len() as f64 * (coordinate_length as f64 / 8.0);
        let mut data = Vec::with_capacity(capacity as usize);
        for cell in cells {
            buffer |= cell << 31 - bit_count;
            bit_count += coordinate_length;
            while bit_count >= 32 {
                let byte = buffer & mask;
                bit_count -= BYTE;
                buffer <<= BYTE;

                data.push((byte >> 24) as u8);
            }
        }
        while buffer != 0 {
            let byte = (buffer & 0xFF000000) >> 24;
            data.push(byte as u8);
            buffer <<= BYTE;
        }
        data
    }

    pub fn encode_header(&self, payload_length: u32, fn_call_id: u8, sum: &mut u32) -> Vec<u8> {
        let mut data = Vec::with_capacity(HEADER_SIZE_BYTES);

        data.push(self.header.version);
        data.push(fn_call_id);
        data.push_u16_to_u8s(self.header.msg_id);
        data.push_u16_to_u8s(self.header.image_size);
        data.push_u32_to_u8s(payload_length);

        self.ones_complement_sum(sum, self.header.version as u16);
        self.ones_complement_sum(sum, fn_call_id as u16);
        self.ones_complement_sum(sum, self.header.msg_id);
        self.ones_complement_sum(sum, self.header.image_size);
        self.ones_complement_sum(sum, (payload_length >> 16) as u16);
        self.ones_complement_sum(sum, payload_length as u16);
        return data;
    }

    pub fn encode_cells_from_set(
        &self,
        client: &TcpStream,
        cells: IndexSet<u32>,
        function_call_id: u8,
    ) {
        let (coordinate_length, _) =
            Coordinates::calc_coord_len_and_offset(self.header.image_size as u32);

        // payload_length is length of alivecells in bytes + header size in bytes
        let payload = self.encode_payload_from_set(cells, coordinate_length as usize);

        let payload_length = payload.len() as u32 + HEADER_SIZE_BYTES as u32;
        let mut sum: u32 = 0;
        
        let mut header = self.encode_header(payload_length, function_call_id, &mut sum);

        for i in 0..payload.len() / 2 {
            let word = ((payload[i * 2] as u16) << 8) | (payload[i * 2 + 1] as u16);
            self.ones_complement_sum(&mut sum, word);
        }
        // Handle odd-length payload
        if payload.len() % 2 == 1 {
            self.ones_complement_sum(&mut sum, (payload[payload.len() - 1] as u16) << 8);
        }

        let checksum = !(sum as u16);
        header.push_u16_to_u8s(checksum);

        // write HEADER_SIZE number of bites
        // write payload length number of bites
    }

    /// generates mask of left aligned 1's where there are `coordinate_length` number of 1's
    fn generate_mask(&self, coordinate_length: u32) -> u32 {
        if coordinate_length > 32 {
            panic!("coordinate length must be less than or equal to 32");
        }
        let mask = !0u32;
        mask << (32 - coordinate_length)
    }

    /// returns the limit the decoder should wait for the bit count to reach before continuing to next chunk
    fn limit(&self, coordinate_length: u32) -> usize {
        if coordinate_length > 24 {
            32
        } else if coordinate_length > 16 {
            24
        } else if coordinate_length > 8 {
            16
        } else {
            8
        }
    }

    fn ones_complement_sum(&self, sum: &mut u32, word: u16) {
        *sum += word as u32;
        if *sum > 0xFFFF {
            *sum = (*sum & 0xFFFF) + 1;
        }
    }
}

trait Data {
    fn push_u16_to_u8s(&mut self, num: u16);
    fn push_u24_to_u8s(&mut self, num: u32);
    fn push_u32_to_u8s(&mut self, num: u32);
}

impl Data for Vec<u8> {
    fn push_u16_to_u8s(&mut self, num: u16) {
        self.push((num >> 8) as u8);
        self.push(num as u8);
    }

    fn push_u24_to_u8s(&mut self, num: u32) {
        self.push((num >> 16) as u8);
        self.push((num >> 8) as u8);
        self.push(num as u8);
    }

    fn push_u32_to_u8s(&mut self, num: u32) {
        self.push((num >> 24) as u8);
        self.push((num >> 16) as u8);
        self.push((num >> 8) as u8);
        self.push(num as u8);
    }
}
