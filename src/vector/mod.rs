use std::ops;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point2([f32; 2]);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector2([f32; 2]);

impl Point2 {
    pub fn new(x: f32, y: f32) -> Point2 {
        Point2 { 0: [x, y] }
    }

    pub fn get_x(&self) -> f32 {
        self.0[0]
    }

    pub fn get_y(&self) -> f32 {
        self.0[1]
    }
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Vector2 {
        Vector2 { 0: [x, y] }
    }

    pub fn dot(&self, other: &Vector2) -> f32 {
        self.0[0] * other.0[0] + self.0[1] * other.0[1]
    }

    pub fn cross(&self, other: &Vector2) -> f32 {
        self.0[0] * other.0[1] - other.0[0] * self.0[1]
    }

    pub fn length(&self) -> f32 {
        ((self.0[0] * self.0[0]) + (self.0[1] * self.0[1])).sqrt()
    }

    pub fn unit(&self) -> Vector2 {
        let new_x = self.0[0] / self.length();
        let new_y = self.0[1] / self.length();
        Vector2 { 0: [new_x, new_y] }
    }

    pub fn to_unit(&mut self) {
        let new_x = self.0[0] / self.length();
        let new_y = self.0[1] / self.length();

        self.0[0] = new_x;
        self.0[1] = new_y;
    }

    pub fn get_x(&self) -> f32 {
        self.0[0]
    }

    pub fn get_y(&self) -> f32 {
        self.0[1]
    }
}

impl ops::Add<Vector2> for Vector2 {
    type Output = Vector2;

    fn add(self, rhs: Vector2) -> Vector2 {
        Vector2 {
            0: [self.0[0] + rhs.0[0], self.0[1] + rhs.0[1]],
        }
    }
}

impl ops::Neg for Vector2 {
    type Output = Vector2;

    fn neg(self) -> Vector2 {
        Vector2 {
            0: [-self.0[0], -self.0[1]],
        }
    }
}

impl ops::Sub<Vector2> for Vector2 {
    type Output = Vector2;

    fn sub(self, rhs: Vector2) -> Vector2 {
        self + (-rhs)
    }
}

impl ops::Mul<f32> for Vector2 {
    type Output = Vector2;

    fn mul(self, rhs: f32) -> Vector2 {
        Vector2 {
            0: [self.0[0] * rhs, self.0[1] * rhs],
        }
    }
}

impl ops::Div<f32> for Vector2 {
    type Output = Vector2;

    fn div(self, rhs: f32) -> Vector2 {
        Vector2 {
            0: [self.0[0] / rhs, self.0[1] / rhs],
        }
    }
}

impl ops::Add<Vector2> for Point2 {
    type Output = Point2;

    fn add(self, rhs: Vector2) -> Point2 {
        Point2 {
            0: [self.0[0] + rhs.0[0], self.0[1] + rhs.0[1]],
        }
    }
}

impl ops::Sub<Point2> for Point2 {
    type Output = Vector2;

    fn sub(self, rhs: Point2) -> Vector2 {
        Vector2 {
            0: [self.0[0] - rhs.0[0], self.0[1] - rhs.0[1]],
        }
    }
}

impl ops::Sub<Vector2> for Point2 {
    type Output = Point2;

    fn sub(self, rhs: Vector2) -> Point2 {
        self + (-rhs)
    }
}
