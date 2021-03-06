use cgmath::{Point3, Vector3, InnerSpace, Zero};


pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Camera {
    pos: Point3<f32>,
    front: Vector3<f32>,
    movement: [f32; 4],
}

impl Camera {
    pub fn new(pos: Point3<f32>, to: Point3<f32>) -> Camera {
        Camera {
            pos,
            front: to - pos,
            movement: [0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn movement_vector(&self) -> Option<Vector3<f32>> {
        let v = Vector3::new(
            self.movement[2] - self.movement[3],
            self.movement[0] - self.movement[1],
            0.0,
        );
        if !v.is_zero() {
            Some(v.normalize())
        } else {
            None
        }
    }

    pub fn prep_move(&mut self, dir: Direction, m: bool) {
        let index = match dir {
            Direction::Up => 0,
            Direction::Down => 1,
            Direction::Left => 3,
            Direction::Right => 2,
        };
        self.movement[index] = if m { 1.0 } else { 0.0 };
    }

    pub fn move_at(&mut self, speed: f32) {
        if let Some(v) = self.movement_vector() {
            self.pos += v * speed;
        }
    }

    pub fn pos(&self) -> Point3<f32> {
        self.pos
    }

    pub fn looking_at(&self) -> Point3<f32> {
        self.pos + self.front
    }
}
