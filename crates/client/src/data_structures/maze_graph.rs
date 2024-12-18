use std::collections::{HashMap, HashSet};

use shared::{maze::Cell, radar::CellType};

#[derive(Debug)]
pub struct MazeCell {
    pub cell_type: CellType,
    pub neighbors: HashSet<Cell>,
}
#[derive(Debug)]
pub struct MazeGraph {
    cell_map: HashMap<Cell, MazeCell>,
}

impl Default for MazeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl MazeGraph {
    pub fn new() -> Self {
        Self { cell_map: HashMap::new() }
    }
    pub fn contains(&self, cell: &Cell) -> bool {
        self.cell_map.contains_key(cell)
    }

    pub fn add(&mut self, cell: Cell, cell_type: CellType) {
        self.cell_map.insert(cell, MazeCell { cell_type, neighbors: HashSet::new() });
    }

    pub fn add_neighbor(&mut self, cell: &Cell, neighbor: &Cell) {
        let maze_cell: &mut MazeCell = self.cell_map.get_mut(cell).unwrap();
        maze_cell.neighbors.insert(*neighbor);
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_hash_set() {
        let mut f: HashSet<Cell> = HashSet::new();
        let c: Cell = Cell { row: -1, column: 1 };
        let d: Cell = Cell { row: -1, column: 1 };
        f.insert(c);
        assert!(f.contains(&d));
    }
}
