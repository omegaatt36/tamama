use crate::config::Config;
use rand::{thread_rng, Rng};

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn random(max_x: f32, max_y: f32) -> Self {
        let mut rng = thread_rng();
        Self {
            x: rng.gen_range(0.0..max_x),
            y: rng.gen_range(0.0..max_y),
        }
    }

    pub fn random_unit() -> Self {
        let mut rng = thread_rng();
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        Self {
            x: angle.cos(),
            y: angle.sin(),
        }
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn magnitude_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self {
                x: self.x / mag,
                y: self.y / mag,
            }
        } else {
            *self
        }
    }

    pub fn limit(&self, max: f32) -> Self {
        if self.magnitude_squared() > max * max {
            self.normalize() * max
        } else {
            *self
        }
    }

    pub fn distance_to(&self, other: &Vec2) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, scalar: f32) -> Vec2 {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl std::ops::Div<f32> for Vec2 {
    type Output = Vec2;

    fn div(self, scalar: f32) -> Vec2 {
        Vec2 {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, other: Vec2) {
        self.x += other.x;
        self.y += other.y;
    }
}

#[derive(Debug, Clone)]
pub struct Boid {
    pub position: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub is_leader: bool,
}

impl Boid {
    pub fn new(config: &Config) -> Self {
        Self {
            position: Vec2::random(config.width, config.height),
            velocity: Vec2::random_unit() * (config.max_speed * 0.5),
            acceleration: Vec2::zero(),
            is_leader: false,
        }
    }

    pub fn new_leader(config: &Config) -> Self {
        Self {
            position: Vec2 {
                x: config.width * 0.1,
                y: config.height * 0.5,
            },
            velocity: Vec2 {
                x: config.max_speed * 0.6,
                y: 0.0,
            },
            acceleration: Vec2::zero(),
            is_leader: true,
        }
    }

    pub fn update(&mut self, config: &Config) {
        self.velocity += self.acceleration;
        self.velocity = self.velocity.limit(config.max_speed);
        self.position += self.velocity;
        self.acceleration = Vec2::zero();

        self.bounce_off_boundaries(config);
    }

    pub fn apply_force(&mut self, force: Vec2) {
        self.acceleration += force;
    }

    fn bounce_off_boundaries(&mut self, config: &Config) {
        let margin = 1.0;

        if self.position.x < margin {
            self.position.x = margin;
            self.velocity.x = self.velocity.x.abs();
        } else if self.position.x > config.width - margin {
            self.position.x = config.width - margin;
            self.velocity.x = -self.velocity.x.abs();
        }

        if self.position.y < margin {
            self.position.y = margin;
            self.velocity.y = self.velocity.y.abs();
        } else if self.position.y > config.height - margin {
            self.position.y = config.height - margin;
            self.velocity.y = -self.velocity.y.abs();
        }
    }

    pub fn get_direction_char(&self) -> char {
        if self.is_leader {
            return '★';
        }

        let angle = self.velocity.y.atan2(self.velocity.x);
        let pi = std::f32::consts::PI;

        if angle >= -pi / 8.0 && angle < pi / 8.0 {
            '>'
        } else if angle >= pi / 8.0 && angle < 3.0 * pi / 8.0 {
            '\\'
        } else if angle >= 3.0 * pi / 8.0 && angle < 5.0 * pi / 8.0 {
            'v'
        } else if angle >= 5.0 * pi / 8.0 && angle < 7.0 * pi / 8.0 {
            '/'
        } else if angle >= 7.0 * pi / 8.0 || angle < -7.0 * pi / 8.0 {
            '<'
        } else if angle >= -7.0 * pi / 8.0 && angle < -5.0 * pi / 8.0 {
            '/'
        } else if angle >= -5.0 * pi / 8.0 && angle < -3.0 * pi / 8.0 {
            '^'
        } else {
            '\\'
        }
    }
}
