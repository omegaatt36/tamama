use crate::boid::{Boid, Vec2};
use crate::config::Config;
use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy)]
pub enum PatrolDirection {
    ToRight,
    ToLeft,
}

pub struct LeaderBird {
    pub boid_index: usize,
    pub direction: PatrolDirection,
    pub target_x: f32,
    pub sine_time: f32,
    pub sine_frequency: f32,
    pub sine_amplitude: f32,
}

impl LeaderBird {
    pub fn new(boid_index: usize, config: &Config) -> Self {
        Self {
            boid_index,
            direction: PatrolDirection::ToRight,
            target_x: config.width * 0.9,
            sine_time: 0.0,
            sine_frequency: 0.02,                // Sine wave frequency
            sine_amplitude: config.height * 0.3, // Sine wave amplitude
        }
    }
}

pub struct Simulation {
    pub boids: Vec<Boid>,
    pub config: Config,
    pub leader: Option<LeaderBird>,
}

impl Simulation {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let config = Config::default();
        let mut boids = Vec::new();

        // Create leader bird
        boids.push(Boid::new_leader(&config));

        // Create other boids
        for _ in 1..config.num_boids {
            boids.push(Boid::new(&config));
        }

        let leader = Some(LeaderBird::new(0, &config));

        Self {
            boids,
            config,
            leader,
        }
    }

    pub fn new_with_size(terminal_size: Rect) -> Self {
        let config = Config::with_terminal_size(terminal_size);
        let mut boids = Vec::new();

        // Create leader bird
        boids.push(Boid::new_leader(&config));

        // Create other boids
        for _ in 1..config.num_boids {
            boids.push(Boid::new(&config));
        }

        let leader = Some(LeaderBird::new(0, &config));

        Self {
            boids,
            config,
            leader,
        }
    }

    pub fn update(&mut self) {
        // Update leader logic
        self.update_leader_state();

        let mut forces = Vec::new();

        for i in 0..self.boids.len() {
            if self.boids[i].is_leader {
                // Leader bird uses special patrol force
                let patrol_force = self.leader_patrol_force(i);
                forces.push(patrol_force);
            } else {
                // Regular boids use Boids rules + follow leader
                let separation = self.separation(i);
                let alignment = self.alignment(i);
                let cohesion = self.cohesion(i);
                let follow_leader = self.follow_leader_force(i);

                let total_force = separation * self.config.separation_weight
                    + alignment * self.config.alignment_weight
                    + cohesion * self.config.cohesion_weight
                    + follow_leader * 1.5; // Higher weight for following leader

                forces.push(total_force);
            }
        }

        for (i, force) in forces.into_iter().enumerate() {
            self.boids[i].apply_force(force);
            self.boids[i].update(&self.config);
        }
    }

    fn separation(&self, index: usize) -> Vec2 {
        let current_boid = &self.boids[index];
        let mut steer = Vec2::zero();
        let mut count = 0;

        for (i, other) in self.boids.iter().enumerate() {
            if i == index {
                continue;
            }

            let distance = current_boid.position.distance_to(&other.position);

            if distance > 0.0 && distance < self.config.separation_radius {
                let mut diff = current_boid.position - other.position;
                diff = diff.normalize() / distance;
                steer += diff;
                count += 1;
            }
        }

        if count > 0 {
            steer = steer / count as f32;
            steer = steer.normalize() * self.config.max_speed;
            steer = steer - current_boid.velocity;
            steer = steer.limit(self.config.max_force);
        }

        steer
    }

    fn alignment(&self, index: usize) -> Vec2 {
        let current_boid = &self.boids[index];
        let mut sum = Vec2::zero();
        let mut count = 0;

        for (i, other) in self.boids.iter().enumerate() {
            if i == index {
                continue;
            }

            let distance = current_boid.position.distance_to(&other.position);

            if distance > 0.0 && distance < self.config.alignment_radius {
                sum += other.velocity;
                count += 1;
            }
        }

        if count > 0 {
            sum = sum / count as f32;
            sum = sum.normalize() * self.config.max_speed;
            let steer = sum - current_boid.velocity;
            steer.limit(self.config.max_force)
        } else {
            Vec2::zero()
        }
    }

    fn cohesion(&self, index: usize) -> Vec2 {
        let current_boid = &self.boids[index];
        let mut sum = Vec2::zero();
        let mut count = 0;

        for (i, other) in self.boids.iter().enumerate() {
            if i == index {
                continue;
            }

            let distance = current_boid.position.distance_to(&other.position);

            if distance > 0.0 && distance < self.config.cohesion_radius {
                sum += other.position;
                count += 1;
            }
        }

        if count > 0 {
            sum = sum / count as f32;
            self.seek(current_boid, sum)
        } else {
            Vec2::zero()
        }
    }

    fn seek(&self, boid: &Boid, target: Vec2) -> Vec2 {
        let desired = target - boid.position;
        let desired = desired.normalize() * self.config.max_speed;
        let steer = desired - boid.velocity;
        steer.limit(self.config.max_force)
    }

    pub fn reset(&mut self) {
        self.boids.clear();

        // Re-create leader bird
        self.boids.push(Boid::new_leader(&self.config));

        // Re-create other boids
        for _ in 1..self.config.num_boids {
            self.boids.push(Boid::new(&self.config));
        }

        // Reset leader state
        self.leader = Some(LeaderBird::new(0, &self.config));
    }

    pub fn adjust_boid_count_for_size(&mut self, terminal_size: Rect) {
        let new_config = Config::with_terminal_size(terminal_size);
        let target_count = new_config.num_boids;
        let current_count = self.boids.len();

        // Update config (other parameters will also be updated except boid count)
        self.config = new_config;

        if target_count > current_count {
            // Increase boid count
            for _ in current_count..target_count {
                self.boids.push(Boid::new(&self.config));
            }
        } else if target_count < current_count {
            // Decrease boid count, but protect leader bird (index 0)
            if target_count > 0 {
                self.boids.truncate(target_count);
            }
        }

        // Update leader target
        if let Some(ref mut leader) = self.leader {
            leader.target_x = match leader.direction {
                PatrolDirection::ToRight => self.config.width * 0.9,
                PatrolDirection::ToLeft => self.config.width * 0.1,
            };
            leader.sine_amplitude = self.config.height * 0.3;
        }
    }

    // Leader bird related methods
    fn update_leader_state(&mut self) {
        if let Some(ref mut leader) = self.leader {
            if leader.boid_index < self.boids.len() {
                let leader_boid = &self.boids[leader.boid_index];

                // Update sine wave time
                leader.sine_time += leader.sine_frequency;

                // Check if reached boundary and need to switch direction
                match leader.direction {
                    PatrolDirection::ToRight => {
                        if leader_boid.position.x >= self.config.width * 0.9 {
                            leader.direction = PatrolDirection::ToLeft;
                            leader.target_x = self.config.width * 0.1;
                        }
                    }
                    PatrolDirection::ToLeft => {
                        if leader_boid.position.x <= self.config.width * 0.1 {
                            leader.direction = PatrolDirection::ToRight;
                            leader.target_x = self.config.width * 0.9;
                        }
                    }
                }
            }
        }
    }

    fn leader_patrol_force(&self, index: usize) -> Vec2 {
        if let Some(ref leader) = self.leader {
            if index == leader.boid_index {
                let boid = &self.boids[index];

                // Calculate sine wave trajectory target position
                let center_y = self.config.height * 0.5;
                let sine_y = center_y + (leader.sine_time.sin() * leader.sine_amplitude);

                let target = Vec2 {
                    x: leader.target_x,
                    y: sine_y.max(0.0).min(self.config.height),
                };

                let desired = target - boid.position;
                if desired.magnitude() > 0.0 {
                    let desired = desired.normalize() * self.config.max_speed;
                    let steer = desired - boid.velocity;
                    return steer.limit(self.config.max_force);
                }
            }
        }
        Vec2::zero()
    }

    fn follow_leader_force(&self, index: usize) -> Vec2 {
        if let Some(ref leader) = self.leader {
            let leader_boid = &self.boids[leader.boid_index];
            let current_boid = &self.boids[index];

            // Calculate desired position for following leader (behind the leader)
            let follow_distance = 8.0;
            let offset = Vec2 {
                x: -leader_boid.velocity.normalize().x * follow_distance,
                y: -leader_boid.velocity.normalize().y * follow_distance,
            };

            let target_position = leader_boid.position + offset;
            let desired = target_position - current_boid.position;

            if desired.magnitude() > 0.0 {
                let desired = desired.normalize() * self.config.max_speed * 0.8;
                let steer = desired - current_boid.velocity;
                return steer.limit(self.config.max_force * 0.7);
            }
        }
        Vec2::zero()
    }
}
