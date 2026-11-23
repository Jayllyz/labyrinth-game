use crate::{maze::Cell, messages::Direction};

/// Where a player faces. Players start facing North. The client never learns its real
/// heading, so on its own map North is simply the heading it started with.
///
/// In `delta` and `rotate`, a `Cell`'s `row` grows towards East and its `column` towards
/// South.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Heading {
    North,
    East,
    South,
    West,
}

impl Heading {
    pub const CLOCKWISE: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    /// The cell offset of one step this way.
    pub fn delta(self) -> Cell {
        match self {
            Self::North => Cell { row: 0, column: -1 },
            Self::East => Cell { row: 1, column: 0 },
            Self::South => Cell { row: 0, column: 1 },
            Self::West => Cell { row: -1, column: 0 },
        }
    }

    /// The heading whose step is `delta`, if it is one step.
    pub fn from_delta(delta: Cell) -> Option<Self> {
        Self::CLOCKWISE.into_iter().find(|heading| heading.delta() == delta)
    }

    /// Clockwise degrees from North.
    pub fn degrees(self) -> f32 {
        f32::from(self.quarters()) * 90.0
    }

    /// The heading after a relative move.
    pub fn turn(self, direction: Direction) -> Self {
        let quarters = match direction {
            Direction::Front => 0,
            Direction::Right => 1,
            Direction::Back => 2,
            Direction::Left => 3,
        };
        Self::CLOCKWISE[usize::from((self.quarters() + quarters) % 4)]
    }

    /// The relative move that turns this heading into `target`.
    pub fn direction_to(self, target: Self) -> Direction {
        match (target.quarters() + 4 - self.quarters()) % 4 {
            0 => Direction::Front,
            1 => Direction::Right,
            2 => Direction::Back,
            _ => Direction::Left,
        }
    }

    /// Turns an offset seen facing North into the same offset seen facing this way.
    pub fn rotate(self, offset: Cell) -> Cell {
        (0..self.quarters()).fold(offset, |cell, _| Cell { row: -cell.column, column: cell.row })
    }

    /// Position in `CLOCKWISE`, for arrays indexed by heading.
    pub fn index(self) -> usize {
        usize::from(self.quarters())
    }

    fn quarters(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turning_then_asking_the_way_back_agree() {
        for heading in Heading::CLOCKWISE {
            for direction in [Direction::Front, Direction::Right, Direction::Back, Direction::Left]
            {
                assert_eq!(heading.direction_to(heading.turn(direction)), direction);
            }
        }
    }

    #[test]
    fn test_rotating_a_step_north_gives_the_heading_step() {
        for heading in Heading::CLOCKWISE {
            assert_eq!(heading.rotate(Heading::North.delta()), heading.delta());
        }
    }
}
