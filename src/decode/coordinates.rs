pub struct Coordinates {}

impl Coordinates {
    /// returns `(coordinate_length, offset)`
    ///
    /// `coordinate_length` is the length of the combined `xy` coordinate.
    /// - For 16x16 this will be 8 bits.
    /// - For 64x64 this will be 12 bits.
    /// - For 512x512 this will be 18 bits.
    ///
    /// `offset` is the remaining number of bits in the `u32` type that are unused by the coordinate
    /// - For 16x16 this will be 24 bits.
    /// - For 64x64 this will be 20 bits.
    /// - For 512x512 this will be 14 bits.
    pub fn calc_coord_len_and_offset(image_size: u32) -> (u32, u32) {
        let coordinate_length = || -> u32 {
            let mask: u32 = 1;
            let mut size = 0;
            for i in 0..32 as u32 {
                if image_size & (mask << i) > 0 {
                    size = i;
                }
            }
            return size * 2;
        }();
        let offset = 32 - coordinate_length;
        return (coordinate_length, offset);
    }

    /// generates mask of left aligned 1's where there are `coordinate_length` number of 1's
    fn generate_mask(coordinate_length: u32) -> u32 {
        if coordinate_length > 32 {
            panic!("coordinate length must be less than or equal to 32");
        }
        let mask = !0u32;
        mask << (32 - coordinate_length)
    }

    /// returns the limit the decoder should wait for the bit count to reach before continuing to next chunk
    fn limit(coordinate_length: u32) -> usize {
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
}