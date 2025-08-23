use ratatui::layout::Rect;

pub struct Config {
    pub width: f32,
    pub height: f32,
    pub num_boids: usize,
    pub max_speed: f32,
    pub max_force: f32,
    pub separation_radius: f32,
    pub alignment_radius: f32,
    pub cohesion_radius: f32,
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
}

impl Config {
    pub fn with_terminal_size(terminal_size: Rect) -> Self {
        // Calculate simulation area (75% for main canvas)
        let canvas_width = (terminal_size.width as f32 * 0.75).max(20.0);
        let canvas_height = (terminal_size.height as f32).max(10.0);
        
        // Dynamically calculate boid count based on area
        let area = canvas_width * canvas_height;
        let density_factor = 0.008; // Approximately 1 boid per 125 characters
        let num_boids = (area * density_factor)
            .max(15.0)    // Minimum 15 boids
            .min(100.0)   // Maximum 100 boids
            as usize;
        
        // Adjust parameters based on boid density
        let boid_density = num_boids as f32 / area;
        let density_multiplier = (boid_density * 1000.0).max(0.5).min(2.0);
        
        Self {
            width: canvas_width,
            height: canvas_height,
            num_boids,
            max_speed: 1.5,
            max_force: 0.08,
            separation_radius: 3.0 * density_multiplier,
            alignment_radius: 5.0,
            cohesion_radius: 5.0,
            separation_weight: 2.0,
            alignment_weight: 1.2,
            cohesion_weight: 1.0,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 80.0,
            height: 24.0,
            num_boids: 25,
            max_speed: 1.5,
            max_force: 0.08,
            separation_radius: 3.0,
            alignment_radius: 5.0,
            cohesion_radius: 5.0,
            separation_weight: 2.0,
            alignment_weight: 1.2,
            cohesion_weight: 1.0,
        }
    }
}