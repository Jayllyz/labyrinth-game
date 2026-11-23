//! What teammates share: their secrets, and their maps so each one can reuse what the
//! others explored.
//!
//! Every agent maps the maze from its own start cell. All agents start facing North, so
//! two maps differ only by a shift, which a mutual sighting reveals.

use crate::data_structures::maze_graph::MazeGraph;
use crate::maze_parser::Player;
use shared::{
    maze::Cell,
    radar::{CellType, Radar},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Secrets of the whole team, by agent name. A challenge is answered with all of them.
pub type TeamSecrets = Arc<Mutex<HashMap<String, u128>>>;

/// The last thing each agent published, by agent name.
pub type TeamMaps = Arc<Mutex<HashMap<String, Snapshot>>>;

#[derive(Clone, Default)]
pub struct Team {
    pub secrets: TeamSecrets,
    pub maps: TeamMaps,
}

/// An agent's position, the teammates its last radar showed, and its map, all in its
/// own frame.
#[derive(Clone)]
pub struct Snapshot {
    pub position: Cell,
    /// Offsets from `position` of the teammates seen.
    pub allies_seen: Vec<Cell>,
    pub graph: MazeGraph,
}

/// A shared map takes a few cell sides seen by both before it is trusted.
const MIN_OVERLAP: usize = 4;

/// Offsets from the player of the teammates on the radar.
pub fn allies_seen(radar: &Radar, player: &Player) -> Vec<Cell> {
    radar
        .cells
        .iter()
        .enumerate()
        .filter(|(_, cell)| matches!(cell, CellType::ALLY | CellType::ObjectiveAlly))
        .map(|(index, _)| {
            let (row, column) = ((index % 3) as i16 - 1, (index / 3) as i16 - 1);
            player.heading.rotate(Cell { row, column })
        })
        .collect()
}

/// The shift that brings `other`'s frame into `me`'s, if `me` sees a teammate where
/// `other` stands and `other` sees a teammate where `me` stands, and their maps agree.
pub fn find_shift(me: &Snapshot, other: &Snapshot) -> Option<Cell> {
    me.allies_seen
        .iter()
        .filter(|seen| other.allies_seen.contains(&Cell { row: -seen.row, column: -seen.column }))
        .map(|seen| {
            let at = me.position + *seen;
            Cell { row: at.row - other.position.row, column: at.column - other.position.column }
        })
        .find(|&shift| me.graph.shared_sides(&other.graph, shift).is_some_and(|n| n >= MIN_OVERLAP))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structures::maze_graph::CellStatus;
    use shared::heading::Heading;

    const fn cell(row: i16, column: i16) -> Cell {
        Cell { row, column }
    }

    /// A map of an East-West corridor from `from` to `to`, row being East.
    fn corridor(from: i16, to: i16) -> MazeGraph {
        let mut graph = MazeGraph::new();
        for row in from..=to {
            graph.add(cell(row, 0), CellType::NOTHING);
        }
        for row in from..=to {
            let open: Vec<Heading> = [(row > from, Heading::West), (row < to, Heading::East)]
                .into_iter()
                .filter_map(|(open, heading)| open.then_some(heading))
                .collect();
            graph.wall_in(cell(row, 0), &open);
            if row > from {
                graph.add_neighbor(&cell(row, 0), &cell(row - 1, 0));
            }
            if row < to {
                graph.add_neighbor(&cell(row, 0), &cell(row + 1, 0));
            }
        }
        graph
    }

    fn snapshot(position: Cell, allies_seen: Vec<Cell>, graph: MazeGraph) -> Snapshot {
        Snapshot { position, allies_seen, graph }
    }

    #[test]
    fn test_mutual_sighting_gives_the_shift() {
        // The same corridor, mapped from cells 2 apart: B's frame is A's moved by 2.
        let me = snapshot(cell(1, 0), vec![cell(1, 0)], corridor(0, 4));
        let other = snapshot(cell(0, 0), vec![cell(-1, 0)], corridor(-2, 2));

        assert_eq!(find_shift(&me, &other), Some(cell(2, 0)));
    }

    #[test]
    fn test_one_sided_sighting_gives_no_shift() {
        let me = snapshot(cell(1, 0), vec![cell(1, 0)], corridor(0, 4));
        let other = snapshot(cell(0, 0), Vec::new(), corridor(-2, 2));

        assert_eq!(find_shift(&me, &other), None);
    }

    #[test]
    fn test_maps_that_disagree_give_no_shift() {
        let me = snapshot(cell(1, 0), vec![cell(1, 0)], corridor(0, 4));
        // B's corridor ends where A's goes on.
        let other = snapshot(cell(0, 0), vec![cell(-1, 0)], corridor(-2, 0));

        assert_eq!(find_shift(&me, &other), None);
    }

    #[test]
    fn test_merge_adds_cells_and_dead_ends_but_keeps_the_agent_cell() {
        let mut mine = corridor(0, 2);
        let mut theirs = corridor(0, 4);
        theirs.update_cell_status(cell(4, 0), CellStatus::DeadEnd);
        theirs.update_cell_status(cell(0, 0), CellStatus::DeadEnd);

        mine.absorb(&theirs, cell(0, 0), cell(0, 0));

        assert_eq!(mine.status_counts().total, 5);
        assert_eq!(mine.get_cell_status(cell(4, 0)), Some(CellStatus::DeadEnd));
        assert_eq!(mine.get_cell_status(cell(0, 0)), Some(CellStatus::NotVisited));
        assert!(mine.get_cell(cell(2, 0)).is_some_and(|c| c.neighbors.contains(&cell(3, 0))));
    }
}
