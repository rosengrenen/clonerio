pub struct Grid {
    pub tiles: [[Option<Belt>; 128]; 128],
}

impl Grid {
    pub fn new() -> Self {
        Self {
            tiles: [[None; 128]; 128],
        }
    }

    pub fn place_belt(&mut self, x: isize, y: isize, belt: Belt) {
        let belt = self.calculate_belt_position(x, y, belt);
        // Adjust input of belt in front
        // - -
        //   |
        // front belt direction west/east
        // current belt direction north
        if let (Some(mut front_belt), (front_belt_x, front_belt_y)) =
            self.belt_in_front_of(x, y, belt)
        {
            if (front_belt.input == belt.output.rotate_clockwise()
                || front_belt.input == belt.output.rotate_anti_clockwise())
                && self
                    .belt_behind(front_belt_x, front_belt_y, front_belt)
                    .0
                    .is_none()
            {
                front_belt.input = belt.output.flip();
                self.set_belt_in_front_of(x, y, belt, front_belt);
            } else if front_belt.output == belt.output.rotate_clockwise()
                || front_belt.output == belt.output.rotate_anti_clockwise()
            {
                if front_belt.input == belt.output {
                    front_belt.input = front_belt.output.flip();
                    self.set_belt_in_front_of(x, y, belt, front_belt);
                }
            } else if front_belt.output == belt.output && front_belt.input != belt.output.flip() {
                front_belt.input = front_belt.output.flip();
                self.set_belt_in_front_of(x, y, belt, front_belt);
            }
        }

        self.set_belt(x, y, belt);
    }

    pub fn calculate_belt_position(&self, x: isize, y: isize, mut belt: Belt) -> Belt {
        let (belt_behind, _) = self.belt_behind(x, y, belt);
        if let Some(belt_behind) = belt_behind {
            if belt_behind.output == belt.input.flip() {
                return belt;
            }
        }

        let (left_belt, _) = self.belt_left_of(x, y, belt);
        let (right_belt, _) = self.belt_right_of(x, y, belt);
        if left_belt.is_some() && right_belt.is_some() {
            let left_belt = left_belt.unwrap();
            let right_belt = right_belt.unwrap();

            let left_belt_facing_into = left_belt.output == belt.output.rotate_clockwise();
            let right_belt_facing_into = right_belt.output == belt.output.rotate_anti_clockwise();

            if left_belt_facing_into && !right_belt_facing_into {
                belt.input = left_belt.output.flip();
            } else if !left_belt_facing_into && right_belt_facing_into {
                belt.input = right_belt.output.flip();
            }
        } else if let Some(left_belt) = left_belt {
            let left_belt_facing_into = left_belt.output == belt.output.rotate_clockwise();

            if left_belt_facing_into {
                belt.input = left_belt.output.flip();
            }
        } else if let Some(right_belt) = right_belt {
            let right_belt_facing_into = right_belt.output == belt.output.rotate_anti_clockwise();

            if right_belt_facing_into {
                belt.input = right_belt.output.flip();
            }
        }

        belt
    }

    pub fn clear_tile(&mut self, x: usize, y: usize) {
        self.tiles[y][x] = None;
    }

    pub fn get_belt(&self, x: isize, y: isize) -> Option<Belt> {
        if x >= 0 && x < self.tiles[0].len() as isize && y >= 0 && y < self.tiles.len() as isize {
            self.tiles[y as usize][x as usize]
        } else {
            None
        }
    }

    pub fn set_belt(&mut self, x: isize, y: isize, belt: Belt) {
        if x >= 0 && x < self.tiles[0].len() as isize && y >= 0 && y < self.tiles.len() as isize {
            self.tiles[y as usize][x as usize] = Some(belt);
        }
    }

    fn left_pos(x: isize, y: isize, belt: Belt) -> (isize, isize) {
        match belt.output {
            Direction::West => (x, y - 1),
            Direction::North => (x - 1, y),
            Direction::East => (x, y + 1),
            Direction::South => (x + 1, y),
        }
    }

    fn right_pos(x: isize, y: isize, belt: Belt) -> (isize, isize) {
        match belt.output {
            Direction::West => (x, y + 1),
            Direction::North => (x + 1, y),
            Direction::East => (x, y - 1),
            Direction::South => (x - 1, y),
        }
    }

    fn front_pos(x: isize, y: isize, belt: Belt) -> (isize, isize) {
        match belt.output {
            Direction::West => (x - 1, y),
            Direction::North => (x, y + 1),
            Direction::East => (x + 1, y),
            Direction::South => (x, y - 1),
        }
    }

    fn behind_pos(x: isize, y: isize, belt: Belt) -> (isize, isize) {
        match belt.input {
            Direction::West => (x - 1, y),
            Direction::North => (x, y + 1),
            Direction::East => (x + 1, y),
            Direction::South => (x, y - 1),
        }
    }

    fn belt_left_of(&self, x: isize, y: isize, belt: Belt) -> (Option<Belt>, (isize, isize)) {
        let (x, y) = Self::left_pos(x, y, belt);
        (self.get_belt(x, y), (x, y))
    }

    fn belt_right_of(&self, x: isize, y: isize, belt: Belt) -> (Option<Belt>, (isize, isize)) {
        let (x, y) = Self::right_pos(x, y, belt);
        (self.get_belt(x, y), (x, y))
    }

    fn belt_in_front_of(&self, x: isize, y: isize, belt: Belt) -> (Option<Belt>, (isize, isize)) {
        let (x, y) = Self::front_pos(x, y, belt);
        (self.get_belt(x, y), (x, y))
    }

    fn set_belt_in_front_of(&mut self, x: isize, y: isize, belt: Belt, new_belt: Belt) {
        let (x, y) = Self::front_pos(x, y, belt);
        self.set_belt(x, y, new_belt);
    }

    fn belt_behind(&self, x: isize, y: isize, belt: Belt) -> (Option<Belt>, (isize, isize)) {
        let (x, y) = Self::behind_pos(x, y, belt);
        (self.get_belt(x, y), (x, y))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Belt {
    pub input: Direction,
    pub output: Direction,
}

impl Belt {
    pub fn new() -> Self {
        Self {
            input: Direction::West,
            output: Direction::East,
        }
    }
}

impl Belt {
    pub fn turn(&self) -> Turn {
        let dir = self.input.rotate_clockwise();
        if dir == self.output {
            return Turn::Left;
        }

        let dir = dir.rotate_clockwise();
        if dir == self.output {
            return Turn::Forward;
        }

        let dir = dir.rotate_clockwise();
        if dir == self.output {
            return Turn::Right;
        }

        panic!();
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    West,
    North,
    East,
    South,
}

impl Direction {
    pub fn rotate_clockwise(&self) -> Self {
        match *self {
            Self::West => Self::North,
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
        }
    }

    pub fn rotate_anti_clockwise(&self) -> Self {
        match *self {
            Self::West => Self::South,
            Self::North => Self::West,
            Self::East => Self::North,
            Self::South => Self::East,
        }
    }

    pub fn flip(&self) -> Self {
        match *self {
            Self::West => Self::East,
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Turn {
    Left,
    Forward,
    Right,
}
