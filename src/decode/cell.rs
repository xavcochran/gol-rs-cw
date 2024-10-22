use indexmap::IndexSet;
pub trait Cell {
    fn neighbours(&self, index: usize, image_size: u32) -> usize;
}

impl Cell for IndexSet<u32> {
    fn neighbours(&self, index: usize, image_size: u32) -> usize {
        let mut live_neighbours = 0;
        let xy = self.get_index(index).unwrap();

        let neighbour_positions = vec![
            (xy + 512) % image_size, // right
            (xy - 512) % image_size, // left
            (xy + 1) % image_size,   // up
            (xy - 1) % image_size,   // down
            (xy + 513) % image_size, // right up
            (xy + 511) % image_size, // right down
            (xy - 511) % image_size, // left down
            (xy - 513) % image_size, // left up
        ];

        for &pos in &neighbour_positions {
            if self.contains(&pos) {
                live_neighbours += 1;
            }
        }

        live_neighbours
    }
}
