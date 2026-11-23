use std::collections::{BTreeSet, HashMap, VecDeque};

use shared::{
    heading::Heading,
    maze::Cell,
    radar::{CellType, Passages, Radar},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CellStatus {
    Visited,
    NotVisited,
    DeadEnd,
}

/// How many cells of a map have each status.
#[derive(Debug, PartialEq, Eq)]
pub struct StatusCounts {
    pub total: usize,
    pub visited: usize,
    pub dead_ends: usize,
    pub not_visited: usize,
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
    cell_map: HashMap<Cell, MazeCell>,
    /// Lets callers skip looking for a path to the exit until one is on the map.
    exit_seen: bool,
    undo: Option<Undo>,
}

/// The cells changed since a checkpoint, as they were before, so the changes can be undone.
#[derive(Debug, Clone)]
struct Undo {
    cells: HashMap<Cell, Option<MazeCell>>,
    exit_seen: bool,
}

impl Default for MazeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl MazeGraph {
    pub fn new() -> Self {
        Self { cell_map: HashMap::new(), exit_seen: false, undo: None }
    }

    /// Starts recording changes, so `rollback` can undo them.
    pub fn checkpoint(&mut self) {
        self.undo = Some(Undo { cells: HashMap::new(), exit_seen: self.exit_seen });
    }

    /// Keeps the changes since the checkpoint.
    pub fn commit(&mut self) {
        self.undo = None;
    }

    /// Undoes the changes since the checkpoint.
    pub fn rollback(&mut self) {
        let Some(undo) = self.undo.take() else {
            return;
        };
        for (cell, before) in undo.cells {
            match before {
                Some(before) => self.cell_map.insert(cell, before),
                None => self.cell_map.remove(&cell),
            };
        }
        self.exit_seen = undo.exit_seen;
    }

    /// Records a cell as it is now, the first time it changes after a checkpoint.
    fn save(&mut self, cell: Cell) {
        if let Some(undo) = &mut self.undo {
            undo.cells.entry(cell).or_insert_with(|| self.cell_map.get(&cell).cloned());
        }
    }

    fn cell_mut(&mut self, position: Cell) -> Option<&mut MazeCell> {
        self.save(position);
        self.cell_map.get_mut(&position)
    }

    /// Adds what a radar view shows, seen from `position` facing `heading`.
    pub fn record(&mut self, radar: &Radar, position: Cell, heading: Heading) {
        let directions_mask = radar_offsets(heading);

        for cell_id in 0..radar.cells.len() {
            if radar.cells[cell_id] == CellType::INVALID {
                continue;
            }

            let cell_pos = position + directions_mask[cell_id];

            if !self.contains(&cell_pos) {
                self.add(cell_pos, radar.cells[cell_id].clone());
            }

            for side in Heading::CLOCKWISE {
                let known = match passage(cell_id, side, &radar.horizontal, &radar.vertical) {
                    Passages::OPEN => Side::Open,
                    Passages::WALL => Side::Wall,
                    Passages::UNDEFINED => continue,
                };
                let heading = Heading::from_delta(heading.rotate(side.delta()));
                if let Some(heading) = heading {
                    self.set_side(cell_pos, heading, known);
                }
            }

            let mut neigbors_to_add: Vec<Cell> = Vec::new();
            for (side, neighbor_id) in radar_neighbors(cell_id) {
                if passage(cell_id, side, &radar.horizontal, &radar.vertical) != Passages::OPEN {
                    continue;
                }
                let neighbor = position + directions_mask[neighbor_id];
                if !self.contains(&neighbor) {
                    self.add(neighbor, radar.cells[neighbor_id].clone());
                }
                neigbors_to_add.push(neighbor);
            }

            for neighbor in neigbors_to_add {
                self.add_neighbor(&cell_pos, &neighbor);
                self.add_neighbor(&neighbor, &cell_pos);
            }
        }
    }

    /// The first step of a shortest path from `from` to a cell that `is_target`, never
    /// through `avoid`. With an exit bearing, steps toward it are tried first.
    pub fn first_step(
        &self,
        from: Cell,
        avoid: Option<Cell>,
        exit_bearing: Option<f32>,
        is_target: impl Fn(&MazeCell) -> bool,
    ) -> Option<Cell> {
        let mut came_from: HashMap<Cell, Cell> = HashMap::from([(from, from)]);
        let mut queue = VecDeque::from([from]);

        while let Some(cell) = queue.pop_front() {
            // Trying first steps toward the exit first breaks ties between equally close cells.
            let bearing = exit_bearing.filter(|_| cell == from);
            for next in self.neighbors_toward(cell, bearing) {
                let Some(next_cell) = self.cell_map.get(&next) else {
                    continue;
                };
                if came_from.contains_key(&next) || Some(next) == avoid {
                    continue;
                }
                came_from.insert(next, cell);
                if is_target(next_cell) {
                    let mut step = next;
                    while came_from[&step] != from {
                        step = came_from[&step];
                    }
                    return Some(step);
                }
                queue.push_back(next);
            }
        }
        None
    }

    /// The open neighbors of `from`, those whose direction is closest to the exit bearing
    /// first.
    pub fn neighbors_toward(&self, from: Cell, exit_bearing: Option<f32>) -> Vec<Cell> {
        let Some(cell) = self.cell_map.get(&from) else {
            return Vec::new();
        };
        let mut cells: Vec<Cell> = cell.neighbors.iter().copied().collect();
        let Some(bearing) = exit_bearing else {
            return cells;
        };
        cells.sort_by_cached_key(|cell| {
            let delta = Cell { row: cell.row - from.row, column: cell.column - from.column };
            let Some(heading) = Heading::from_delta(delta) else {
                return u32::MAX;
            };
            let diff = (heading.degrees() - bearing).rem_euclid(360.0);
            diff.min(360.0 - diff) as u32
        });
        cells
    }

    /// The first step toward the exit, once it is on the map.
    pub fn first_step_to_exit(&self, from: Cell, avoid: Option<Cell>) -> Option<Cell> {
        if !self.exit_seen {
            return None;
        }
        self.first_step(from, avoid, None, |cell| cell.cell_type.is_exit())
    }

    /// How many cell sides both maps know, `other` moved by `shift`, or `None` if one
    /// of them disagrees.
    pub fn shared_sides(&self, other: &Self, shift: Cell) -> Option<usize> {
        let mut shared = 0;
        for (cell, theirs) in &other.cell_map {
            let Some(ours) = self.cell_map.get(&(*cell + shift)) else {
                continue;
            };
            if ours.cell_type.is_exit() != theirs.cell_type.is_exit() {
                return None;
            }
            for (a, b) in ours.sides.iter().zip(&theirs.sides) {
                if *a == Side::Unknown || *b == Side::Unknown {
                    continue;
                }
                if a != b {
                    return None;
                }
                shared += 1;
            }
        }
        Some(shared)
    }

    /// Copies what `other` knows, moved by `shift`: new cells, passages and sides, and the
    /// branches it found to be dead ends. `keep` is left as is, so the agent's own cell
    /// keeps its marks.
    pub fn absorb(&mut self, other: &Self, shift: Cell, keep: Cell) {
        for (cell, theirs) in &other.cell_map {
            let target = *cell + shift;
            if target == keep {
                continue;
            }
            if !self.contains(&target) {
                // Players seen there have moved on, so only the exit is worth keeping.
                let cell_type = if theirs.cell_type.is_exit() {
                    CellType::OBJECTIVE
                } else {
                    CellType::NOTHING
                };
                self.add(target, cell_type);
            }
            for neighbor in &theirs.neighbors {
                self.add_neighbor(&target, &(*neighbor + shift));
            }
            for heading in Heading::CLOCKWISE {
                if theirs.side(heading) != Side::Unknown {
                    self.set_side(target, heading, theirs.side(heading));
                }
            }
            if theirs.status == CellStatus::DeadEnd {
                self.update_cell_status(target, CellStatus::DeadEnd);
            }
        }
    }

    /// How many cells have each status.
    pub fn status_counts(&self) -> StatusCounts {
        let count = |status| self.cell_map.values().filter(|cell| cell.status == status).count();
        StatusCounts {
            total: self.cell_map.len(),
            visited: count(CellStatus::Visited),
            dead_ends: count(CellStatus::DeadEnd),
            not_visited: count(CellStatus::NotVisited),
        }
    }

    /// The smallest and largest row and column on the map.
    pub fn bounds(&self) -> (i16, i16, i16, i16) {
        self.cell_map.keys().fold(
            (i16::MAX, i16::MIN, i16::MAX, i16::MIN),
            |(min_row, max_row, min_col, max_col), cell| {
                (
                    min_row.min(cell.row),
                    max_row.max(cell.row),
                    min_col.min(cell.column),
                    max_col.max(cell.column),
                )
            },
        )
    }

    pub fn contains(&self, cell: &Cell) -> bool {
        self.cell_map.contains_key(cell)
    }

    pub fn add(&mut self, cell: Cell, cell_type: CellType) {
        self.save(cell);
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
        if let Some(maze_cell) = self.cell_mut(*cell) {
            maze_cell.neighbors.insert(*neighbor);
        }
    }

    pub fn get_cell(&self, position: Cell) -> Option<&MazeCell> {
        self.cell_map.get(&position)
    }

    pub fn update_cell_status(&mut self, position: Cell, status: CellStatus) {
        if let Some(cell) = self.cell_mut(position) {
            cell.status = status;
        }
    }

    pub fn update_cell_type(&mut self, position: Cell, cell_type: CellType) {
        if let Some(cell) = self.cell_mut(position) {
            cell.cell_type = cell_type;
        }
    }

    pub fn set_side(&mut self, position: Cell, heading: Heading, side: Side) {
        if let Some(cell) = self.cell_mut(position) {
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
        if let Some(cell) = self.cell_mut(position) {
            cell.parent = parent;
        }
    }

    /// The cell's status, or `None` when the cell is not on the map.
    pub fn get_cell_status(&self, position: Cell) -> Option<CellStatus> {
        self.cell_map.get(&position).map(|cell| cell.status.clone())
    }

    pub fn set_visited(&mut self, position: Cell, thread_name: &str) {
        if let Some(cell) = self.cell_mut(position) {
            let count = cell.visited_by.entry(thread_name.to_string()).or_insert(0);
            *count = count.saturating_add(1);
        }
    }
}

/// The radar cells next to `cell_id`, with the side they are on, the radar's top being North.
fn radar_neighbors(cell_id: usize) -> impl Iterator<Item = (Heading, usize)> {
    let (row, column) = (cell_id / 3, cell_id % 3);
    [
        (Heading::North, (row > 0).then(|| cell_id - 3)),
        (Heading::East, (column < 2).then(|| cell_id + 1)),
        (Heading::South, (row < 2).then(|| cell_id + 3)),
        (Heading::West, (column > 0).then(|| cell_id - 1)),
    ]
    .into_iter()
    .filter_map(|(side, neighbor)| neighbor.map(|id| (side, id)))
}

/// The passage on one side of a radar cell, the radar's top being North.
fn passage(
    cell_id: usize,
    side: Heading,
    horizontal: &[Passages],
    vertical: &[Passages],
) -> Passages {
    let passage = match side {
        Heading::North => &horizontal[cell_id],
        Heading::South => &horizontal[cell_id + 3],
        Heading::West => &vertical[cell_id + cell_id / 3],
        Heading::East => &vertical[cell_id + cell_id / 3 + 1],
    };
    passage.clone()
}

/// Map offsets of the radar's 3x3 cells, row by row, as seen facing `heading`.
fn radar_offsets(heading: Heading) -> Vec<Cell> {
    (-1..=1)
        .flat_map(|column| (-1..=1).map(move |row| Cell { row, column }))
        .map(|offset| heading.rotate(offset))
        .collect()
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::maze_parser::Player;
    use shared::radar;
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

        assert_eq!(maze_graph.get_cell(cell).and_then(|c| c.visited_by.get("a")), Some(&2));
    }

    #[test]
    fn test_rollback_undoes_changes_since_the_checkpoint() {
        let mut maze_graph = MazeGraph::new();
        let kept = Cell { row: 0, column: 0 };
        let added = Cell { row: 1, column: 0 };
        maze_graph.add(kept, CellType::NOTHING);

        maze_graph.checkpoint();
        maze_graph.update_cell_status(kept, CellStatus::DeadEnd);
        maze_graph.add(added, CellType::OBJECTIVE);
        maze_graph.rollback();

        assert_eq!(maze_graph.get_cell_status(kept), Some(CellStatus::NotVisited));
        assert!(!maze_graph.contains(&added));
        assert!(!maze_graph.exit_seen);
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
        assert_eq!(maze_graph.status_counts().total, 2);
    }

    #[test]
    fn test_maze_graph_get_cell_status() {
        let mut maze_graph = MazeGraph::new();
        let cell = Cell { row: 0, column: 0 };
        maze_graph.add(cell, CellType::NOTHING);
        maze_graph.update_cell_status(cell, CellStatus::Visited);
        assert_eq!(maze_graph.get_cell_status(cell), Some(CellStatus::Visited));
    }

    #[test]
    fn test_maze_to_graph() {
        let decoded = radar::decode_base64("Hjeikcyc/W8a8pa");
        let data = radar::extract_data(&decoded);

        let mut p = Player {
            position: Cell { row: 0, column: 0 },
            heading: Heading::North,
            exit_bearing: None,
        };
        let mut m = MazeGraph::new();
        m.record(&data, p.position, p.heading);

        p.heading = Heading::East;
        p.position = p.position + Cell { row: 1, column: 0 };

        let decoded = radar::decode_base64("kOuczzGa//apaaa");
        let data = radar::extract_data(&decoded);
        m.record(&data, p.position, p.heading);
    }

    #[test]
    fn test_passage_reads_each_side() {
        let horizontal = vec![
            Passages::WALL,
            Passages::OPEN,
            Passages::WALL,
            Passages::OPEN,
            Passages::WALL,
            Passages::OPEN,
            Passages::WALL,
            Passages::OPEN,
            Passages::WALL,
        ];
        let vertical = vec![
            Passages::WALL,
            Passages::OPEN,
            Passages::WALL,
            Passages::WALL,
            Passages::WALL,
            Passages::OPEN,
        ];

        let walls = |cell_id| {
            Heading::CLOCKWISE
                .iter()
                .filter(|&&side| passage(cell_id, side, &horizontal, &vertical) == Passages::WALL)
                .count()
        };
        assert_eq!(walls(0), 2);
        assert_eq!(walls(1), 2);
    }

    #[test]
    fn test_radar_sides_are_stored_in_map_headings() {
        let mut horizontal = vec![Passages::UNDEFINED; 12];
        horizontal[4] = Passages::WALL;
        let radar = Radar {
            horizontal,
            vertical: vec![Passages::UNDEFINED; 12],
            cells: vec![CellType::NOTHING; 9],
        };
        let mut player = Player::new();
        player.heading = Heading::East;
        let mut graph = MazeGraph::new();

        graph.record(&radar, player.position, player.heading);

        let center = graph.get_cell(player.position).unwrap();
        assert_eq!(center.side(Heading::East), Side::Wall);
        assert_eq!(center.side(Heading::North), Side::Unknown);
    }

    #[test]
    fn test_get_direction_mask() {
        let player = Player::new();
        let mask = radar_offsets(player.heading);
        assert_eq!(mask.len(), 9);
        assert_eq!(mask[4], Cell { row: 0, column: 0 });

        let mut player_right = Player::new();
        player_right.heading = Heading::East;
        let mask_right = radar_offsets(player_right.heading);
        assert_ne!(mask, mask_right);

        let mut player_back = Player::new();
        player_back.heading = Heading::South;
        let mask_back = radar_offsets(player_back.heading);
        assert_ne!(mask, mask_back);
    }

    #[test]
    fn test_mask_facing_east() {
        let mut player = Player::new();
        player.heading = Heading::East;
        let expected_rotation = vec![
            Cell { row: 1, column: -1 },
            Cell { row: 1, column: 0 },
            Cell { row: 1, column: 1 },
            Cell { row: 0, column: -1 },
            Cell { row: 0, column: 0 },
            Cell { row: 0, column: 1 },
            Cell { row: -1, column: -1 },
            Cell { row: -1, column: 0 },
            Cell { row: -1, column: 1 },
        ];

        assert_eq!(radar_offsets(player.heading), expected_rotation);
    }

    #[test]
    fn test_mask_facing_west() {
        let mut player = Player::new();
        player.heading = Heading::West;
        let expected_rotation = vec![
            Cell { row: -1, column: 1 },
            Cell { row: -1, column: 0 },
            Cell { row: -1, column: -1 },
            Cell { row: 0, column: 1 },
            Cell { row: 0, column: 0 },
            Cell { row: 0, column: -1 },
            Cell { row: 1, column: 1 },
            Cell { row: 1, column: 0 },
            Cell { row: 1, column: -1 },
        ];

        assert_eq!(radar_offsets(player.heading), expected_rotation);
    }

    #[test]
    fn test_bounds() {
        let mut graph = MazeGraph::new();
        assert_eq!(graph.bounds(), (i16::MAX, i16::MIN, i16::MAX, i16::MIN));

        graph.add(Cell { row: 0, column: 0 }, CellType::NOTHING);
        assert_eq!(graph.bounds(), (0, 0, 0, 0));

        graph.add(Cell { row: -1, column: 2 }, CellType::NOTHING);
        graph.add(Cell { row: 3, column: -2 }, CellType::NOTHING);
        assert_eq!(graph.bounds(), (-1, 3, -2, 2));
    }
}
