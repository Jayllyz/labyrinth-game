use std::collections::{BTreeSet, HashMap};

use shared::{heading::Heading, maze::Cell, radar::CellType};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CellStatus {
    Visited,
    NotVisited,
    DeadEnd,
}

/// What is known of one side of a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Side {
    #[default]
    Unknown,
    Open,
    Wall,
}

#[derive(Debug, Clone)]
pub struct MazeCell {
    pub cell_type: CellType,
    /// Ordered, so the same map always gives the same choices.
    pub neighbors: BTreeSet<Cell>,
    pub status: CellStatus,
    /// Indexed by heading, clockwise from North.
    pub sides: [Side; 4],
    pub parent: Cell,
    pub visited_by: HashMap<String, u8>,
}

impl MazeCell {
    pub fn walls(&self) -> usize {
        self.sides.iter().filter(|&&side| side == Side::Wall).count()
    }

    pub fn side(&self, heading: Heading) -> Side {
        self.sides[heading.index()]
    }

    pub fn new(cell_type: CellType) -> Self {
        Self {
            cell_type,
            neighbors: BTreeSet::new(),
            status: CellStatus::NotVisited,
            sides: [Side::Unknown; 4],
            parent: Cell { row: 0, column: 0 },
            visited_by: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MazeGraph {
    pub cell_map: HashMap<Cell, MazeCell>,
    /// Lets callers skip looking for a path to the exit until one is on the map.
    pub exit_seen: bool,
}

impl Default for MazeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl MazeGraph {
    pub fn new() -> Self {
        Self { cell_map: HashMap::new(), exit_seen: false }
    }

    pub fn contains(&self, cell: &Cell) -> bool {
        self.cell_map.contains_key(cell)
    }

    pub fn add(&mut self, cell: Cell, cell_type: CellType) {
        self.exit_seen |= cell_type.is_exit();
        self.cell_map.insert(
            cell,
            MazeCell {
                cell_type,
                neighbors: BTreeSet::new(),
                status: CellStatus::NotVisited,
                parent: cell,
                sides: [Side::Unknown; 4],
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

    pub fn update_cell_type(&mut self, position: Cell, cell_type: CellType) {
        if let Some(cell) = self.cell_map.get_mut(&position) {
            cell.cell_type = cell_type;
        }
    }

    pub fn set_side(&mut self, position: Cell, heading: Heading, side: Side) {
        if let Some(cell) = self.cell_map.get_mut(&position) {
            cell.sides[heading.index()] = side;
        }
    }

    /// Marks every side of a cell but `open` as a wall.
    pub fn wall_in(&mut self, position: Cell, open: &[Heading]) {
        for heading in Heading::CLOCKWISE {
            let side = if open.contains(&heading) { Side::Open } else { Side::Wall };
            self.set_side(position, heading, side);
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

    /// The cell's status, or `None` when the cell is not on the map.
    pub fn get_cell_status(&self, position: Cell) -> Option<CellStatus> {
        self.cell_map.get(&position).map(|cell| cell.status.clone())
    }

    pub fn set_visited(&mut self, position: Cell, thread_name: &str) {
        if let Some(cell) = self.cell_map.get_mut(&position) {
            let count = cell.visited_by.entry(thread_name.to_string()).or_insert(0);
            *count = count.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_hash_set() {
        let mut f: HashSet<Cell> = HashSet::new();
        let c: Cell = Cell { row: -1, column: 1 };
        let d: Cell = Cell { row: -1, column: 1 };
        f.insert(c);
        assert!(f.contains(&d));
    }

    #[test]
    fn test_set_visited_counts_each_agent_visit() {
        let mut maze_graph = MazeGraph::new();
        let cell = Cell { row: 0, column: 0 };
        maze_graph.add(cell, CellType::NOTHING);

        maze_graph.set_visited(cell, "a");
        maze_graph.set_visited(cell, "a");

        assert_eq!(maze_graph.cell_map[&cell].visited_by.get("a"), Some(&2));
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
        maze_graph.update_cell_status(cell, CellStatus::Visited);
        let maze_cell = maze_graph.get_cell(cell).unwrap();
        assert_eq!(maze_cell.status, CellStatus::Visited);
    }

    #[test]
    fn test_walls_counts_wall_sides() {
        let mut maze_graph = MazeGraph::new();
        let cell = Cell { row: 0, column: 0 };
        maze_graph.add(cell, CellType::NOTHING);
        maze_graph.wall_in(cell, &[Heading::South]);
        let maze_cell = maze_graph.get_cell(cell).unwrap();
        assert_eq!(maze_cell.walls(), 3);
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
        maze_graph.update_cell_status(cell, CellStatus::Visited);
        assert_eq!(maze_graph.get_cell_status(cell), Some(CellStatus::Visited));
    }
}
