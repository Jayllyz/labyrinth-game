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
    pub walls: usize,
    pub parent: Cell,
    pub visited_by: HashMap<String, u8>,
}

impl MazeCell {
    pub fn new(cell_type: CellType) -> Self {
        Self {
            cell_type,
            neighbors: HashSet::new(),
            status: CellStatus::NotVisited,
            walls: 0,
            parent: Cell { row: 0, column: 0 },
            visited_by: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MazeGraph {
    pub cell_map: HashMap<Cell, MazeCell>,
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
                visited_by: HashMap::new(),
            },
        );
    }

    pub fn add_neighbor(&mut self, cell: &Cell, neighbor: &Cell) {
        if let Some(maze_cell) = self.cell_map.get_mut(cell) {
            maze_cell.neighbors.insert(*neighbor);
        }
    }

    pub fn get_cell(&mut self, position: Cell) -> Option<&mut MazeCell> {
        self.cell_map.get_mut(&position)
    }

    pub fn update_cell_status(&mut self, position: Cell, status: CellStatus) {
        if let Some(cell) = self.cell_map.get_mut(&position) {
            cell.status = status;
        }
    }

    pub fn update_walls(&mut self, position: Cell, walls: usize) {
        if let Some(cell) = self.cell_map.get_mut(&position) {
            cell.walls = max(cell.walls, walls);
        }
    }

    pub fn set_parent(&mut self, position: Cell, parent: Cell) {
        if let Some(cell) = self.cell_map.get_mut(&position) {
            cell.parent = parent;
        }
    }

    pub fn get_size(&self) -> usize {
        self.cell_map.len()
    }

    pub fn get_cell_status(&self, position: Cell) -> CellStatus {
        if let Some(cell) = self.cell_map.get(&position) {
            cell.status.clone()
        } else {
            CellStatus::DeadEnd
        }
    }

    pub fn set_visited(&self, position: Cell, thread_name: &str) {
        if let Some(cell) = self.cell_map.get(&position) {
            let mut visited = cell.visited_by.clone();

            if let Some(count) = visited.get_mut(thread_name) {
                *count += 1;
            } else {
                visited.insert(thread_name.to_string(), 1);
            }
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
    fn test_maze_graph_add() {
        let mut maze_graph = MazeGraph::new();
        let cell = Cell { row: 0, column: 0 };
        maze_graph.add(cell, CellType::NOTHING);
        assert!(maze_graph.contains(&cell));
    }

    #[test]
    fn test_maze_graph_add_neighbor() {
        let mut maze_graph = MazeGraph::new();
        let cell = Cell { row: 0, column: 0 };
        let neighbor = Cell { row: 0, column: 1 };
        maze_graph.add(cell, CellType::NOTHING);
        maze_graph.add(neighbor, CellType::NOTHING);
        maze_graph.add_neighbor(&cell, &neighbor);
        let maze_cell = maze_graph.get_cell(cell).unwrap();
        assert!(maze_cell.neighbors.contains(&neighbor));
    }

    #[test]
    fn test_maze_graph_update_cell_status() {
        let mut maze_graph = MazeGraph::new();
        let cell = Cell { row: 0, column: 0 };
        maze_graph.add(cell, CellType::NOTHING);
        maze_graph.update_cell_status(cell, CellStatus::VISITED);
        let maze_cell = maze_graph.get_cell(cell).unwrap();
        assert_eq!(maze_cell.status, CellStatus::VISITED);
    }

    #[test]
    fn test_maze_graph_update_walls() {
        let mut maze_graph = MazeGraph::new();
        let cell = Cell { row: 0, column: 0 };
        maze_graph.add(cell, CellType::NOTHING);
        maze_graph.update_walls(cell, 3);
        let maze_cell = maze_graph.get_cell(cell).unwrap();
        assert_eq!(maze_cell.walls, 3);
    }

    #[test]
    fn test_maze_graph_set_parent() {
        let mut maze_graph = MazeGraph::new();
        let cell = Cell { row: 0, column: 0 };
        let parent = Cell { row: 1, column: 1 };
        maze_graph.add(cell, CellType::NOTHING);
        maze_graph.add(parent, CellType::NOTHING);
        maze_graph.set_parent(cell, parent);
        let maze_cell = maze_graph.get_cell(cell).unwrap();
        assert_eq!(maze_cell.parent, parent);
    }

    #[test]
    fn test_maze_graph_get_size() {
        let mut maze_graph = MazeGraph::new();
        let cell1 = Cell { row: 0, column: 0 };
        let cell2 = Cell { row: 1, column: 1 };
        maze_graph.add(cell1, CellType::NOTHING);
        maze_graph.add(cell2, CellType::NOTHING);
        assert_eq!(maze_graph.get_size(), 2);
    }

    #[test]
    fn test_maze_graph_get_cell_status() {
        let mut maze_graph = MazeGraph::new();
        let cell = Cell { row: 0, column: 0 };
        maze_graph.add(cell, CellType::NOTHING);
        maze_graph.update_cell_status(cell, CellStatus::VISITED);
        assert_eq!(maze_graph.get_cell_status(cell), CellStatus::VISITED);
    }
}
