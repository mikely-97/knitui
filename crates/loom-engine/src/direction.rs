#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Returns (row_delta, col_delta) for this direction.
    pub fn offset(self) -> (i32, i32) {
        match self {
            Direction::Up    => (-1,  0),
            Direction::Down  => ( 1,  0),
            Direction::Left  => ( 0, -1),
            Direction::Right => ( 0,  1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_offsets() {
        assert_eq!(Direction::Up.offset(),    (-1,  0));
        assert_eq!(Direction::Down.offset(),  ( 1,  0));
        assert_eq!(Direction::Left.offset(),  ( 0, -1));
        assert_eq!(Direction::Right.offset(), ( 0,  1));
    }
}
