use std::{
    cmp::max,
    collections::{HashMap, HashSet},
};

use shared::{maze::Cell, radar::CellType};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CellStatus {
    VISITED,
    NotVisited,
    DeadEnd,
}

#[derive(Debug, Clone)]
pub struct MazeCell {
    pub cell_type: CellType,
    pub neighbors: HashSet<Cell>,
    pub status: CellStatus,
    pub walls: u8,
    pub parent: Cell,
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
        self.cell_map.insert(
            cell,
            MazeCell {
                cell_type,
                neighbors: HashSet::new(),
                status: CellStatus::NotVisited,
                parent: cell,
                walls: 0,
            },
        );
    }

    pub fn add_neighbor(&mut self, cell: &Cell, neighbor: &Cell) {
        let maze_cell: &mut MazeCell = self.cell_map.get_mut(cell).unwrap();
        maze_cell.neighbors.insert(*neighbor);
    }

    pub fn get_cell(&mut self, position: Cell) -> Option<&mut MazeCell> {
        self.cell_map.get_mut(&position)
    }

    pub fn update_cell_status(&mut self, position: Cell, status: CellStatus) {
        let Some(cell) = self.cell_map.get_mut(&position) else {
            return;
        };

        cell.status = status;
    }

    pub fn update_walls(&mut self, position: Cell, walls: u8) {
        let Some(cell) = self.cell_map.get_mut(&position) else {
            return;
        };

        cell.walls = max(cell.walls, walls);
    }

    pub fn set_parent(&mut self, position: Cell, parent: Cell) {
        let Some(cell) = self.cell_map.get_mut(&position) else {
            return;
        };

        cell.parent = parent;
    }

    pub fn get_size(&self) -> usize {
        self.cell_map.len()
    }

    pub fn get_cell_status(&self, position: Cell) -> CellStatus {
        let Some(cell) = self.cell_map.get(&position) else {
            return CellStatus::DeadEnd;
        };

        cell.status.clone()
    }

    pub fn merge_maze(
        &self,
        graph_to_merge: &mut MazeGraph,
        translation_vector: Cell,
        rotation_degree: i32,
    ) {
        for (old_cell, _maze_cell) in graph_to_merge.cell_map.clone() {
            let new_cell: Cell;
            if rotation_degree == 180 {
                new_cell = Cell { row: -old_cell.row, column: -old_cell.column };
            } else if rotation_degree == 90 {
                new_cell = Cell { row: old_cell.column, column: -old_cell.row };
            } else if rotation_degree == -90 {
                new_cell = Cell { row: -old_cell.column, column: old_cell.row };
            } else {
                continue;
            }

            let new_cell = new_cell + translation_vector;

            graph_to_merge.update_cell_position(old_cell, new_cell);
        }
    }

    pub fn update_cell_position(&mut self, old_cell: Cell, new_cell: Cell) {
        if let Some(maze_cell) = self.cell_map.remove(&old_cell) {
            for neighbor in &maze_cell.neighbors {
                if let Some(neighbor_cell) = self.cell_map.get_mut(neighbor) {
                    neighbor_cell.neighbors.remove(&old_cell);
                    neighbor_cell.neighbors.insert(new_cell);
                }
            }
            self.cell_map.insert(new_cell, maze_cell);
        }
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

    #[test]
    fn test_rotation_maze() {
        let mut graph_target = MazeGraph::new();

        graph_target.add(Cell { row: 0, column: 0 }, CellType::NOTHING);
        graph_target.add(Cell { row: -1, column: 0 }, CellType::NOTHING);
        graph_target.add(Cell { row: -1, column: 1 }, CellType::NOTHING);

        graph_target.add_neighbor(&Cell { row: 0, column: 0 }, &Cell { row: -1, column: 0 });
        graph_target.add_neighbor(&Cell { row: -1, column: 0 }, &Cell { row: 0, column: 0 });

        graph_target.add_neighbor(&Cell { row: -1, column: 0 }, &Cell { row: -1, column: 1 });
        graph_target.add_neighbor(&Cell { row: -1, column: 1 }, &Cell { row: -1, column: 0 });

        let mut graph_source: MazeGraph = MazeGraph::new();

        graph_source.add(Cell { row: 0, column: 0 }, CellType::NOTHING);
        graph_source.add(Cell { row: 0, column: -1 }, CellType::NOTHING);
        graph_source.add(Cell { row: 0, column: -2 }, CellType::NOTHING);

        graph_source.add_neighbor(&Cell { row: 0, column: 0 }, &Cell { row: 0, column: -1 });
        graph_source.add_neighbor(&Cell { row: 0, column: -1 }, &Cell { row: 0, column: 0 });

        graph_source.add_neighbor(&Cell { row: 0, column: -1 }, &Cell { row: 0, column: -2 });
        graph_source.add_neighbor(&Cell { row: 0, column: -2 }, &Cell { row: 0, column: -1 });

        graph_source.merge_maze(&mut graph_target, Cell { row: 2, column: -1 }, -90);
        print!("{:?}", &graph_target);
    }
}
