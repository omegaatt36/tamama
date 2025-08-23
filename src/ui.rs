use crate::simulation::Simulation;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    widgets::canvas::Canvas,
    Frame,
};
use std::time::{Duration, Instant};

pub struct App {
    pub simulation: Simulation,
    pub paused: bool,
    pub high_fps: bool,
    last_update: Instant,
    frame_count: u32,
    fps_counter: f32,
}

impl App {
    pub fn new(terminal_size: Rect) -> Self {
        Self {
            simulation: Simulation::new_with_size(terminal_size),
            paused: false,
            high_fps: false,
            last_update: Instant::now(),
            frame_count: 0,
            fps_counter: 0.0,
        }
    }

    pub fn update(&mut self) {
        if !self.paused {
            self.simulation.update();
        }
        
        self.frame_count += 1;
        let now = Instant::now();
        if now.duration_since(self.last_update) >= Duration::from_secs(1) {
            self.fps_counter = self.frame_count as f32;
            self.frame_count = 0;
            self.last_update = now;
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn toggle_fps(&mut self) {
        self.high_fps = !self.high_fps;
    }

    pub fn reset(&mut self) {
        self.simulation.reset();
    }

    pub fn render(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(f.size());

        let canvas_area = chunks[0];
        self.update_simulation_bounds(canvas_area);

        self.render_simulation(f, canvas_area);
        self.render_info_panel(f, chunks[1]);
    }

    fn update_simulation_bounds(&mut self, area: Rect) {
        let canvas_width = (area.width.saturating_sub(2)) as f32;
        let canvas_height = (area.height.saturating_sub(2)) as f32;
        
        // Check if size has significant change (avoid frequent adjustments)
        let width_diff = (self.simulation.config.width - canvas_width).abs();
        let height_diff = (self.simulation.config.height - canvas_height).abs();
        
        if width_diff > 5.0 || height_diff > 3.0 {
            // Create a virtual terminal size Rect to recalculate boid count
            let virtual_terminal_size = Rect {
                x: 0,
                y: 0,
                width: (canvas_width / 0.75) as u16, // Reverse calculate terminal width
                height: canvas_height as u16,
            };
            
            self.simulation.adjust_boid_count_for_size(virtual_terminal_size);
        } else {
            // Only update boundaries, don't adjust boid count
            self.simulation.config.width = canvas_width;
            self.simulation.config.height = canvas_height;
        }
    }

    fn render_simulation(&self, f: &mut Frame, area: Rect) {
        let canvas = Canvas::default()
            .block(
                Block::default()
                    .title("Boids Simulation")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
            )
            .x_bounds([0.0, self.simulation.config.width.into()])
            .y_bounds([0.0, self.simulation.config.height.into()])
            .paint(|ctx| {
                for boid in &self.simulation.boids {
                    // Hide leader bird, don't display
                    if boid.is_leader {
                        continue;
                    }
                    
                    let color = if self.paused {
                        Color::Gray
                    } else {
                        Color::Green
                    };
                    
                    ctx.print(
                        boid.position.x.into(),
                        (self.simulation.config.height - boid.position.y).into(), 
                        Span::styled(
                            boid.get_direction_char().to_string(),
                            Style::default().fg(color)
                        )
                    );
                }
            });

        f.render_widget(canvas, area);
    }

    fn render_info_panel(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  
                Constraint::Length(10), 
                Constraint::Min(0),     
            ])
            .split(area);

        self.render_controls(f, chunks[0]);
        self.render_stats(f, chunks[1]);
        self.render_parameters(f, chunks[2]);
    }

    fn render_controls(&self, f: &mut Frame, area: Rect) {
        let status = if self.paused { "PAUSED" } else { "RUNNING" };
        let fps_mode = if self.high_fps { "60 FPS" } else { "30 FPS" };
        
        let text = Text::from(vec![
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                Span::styled(status, Style::default().fg(if self.paused { Color::Red } else { Color::Green })),
            ]),
            Line::from(vec![
                Span::styled("FPS: ", Style::default().fg(Color::Yellow)),
                Span::styled(fps_mode, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
            Line::from(Span::styled("Controls:", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
            Line::from("Space - Pause/Resume"),
            Line::from("F - Toggle FPS"),
            Line::from("R - Reset"),
            Line::from("Q - Quit"),
        ]);

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title("Controls")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
            );

        f.render_widget(paragraph, area);
    }

    fn render_stats(&self, f: &mut Frame, area: Rect) {
        let avg_speed: f32 = self.simulation.boids.iter()
            .map(|b| b.velocity.magnitude())
            .sum::<f32>() / self.simulation.boids.len() as f32;

        let text = Text::from(vec![
            Line::from(vec![
                Span::styled("Boids: ", Style::default().fg(Color::Yellow)),
                Span::styled(self.simulation.boids.len().to_string(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Actual FPS: ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:.1}", self.fps_counter), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Avg Speed: ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:.2}", avg_speed), Style::default().fg(Color::White)),
            ]),
        ]);

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title("Statistics")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
            );

        f.render_widget(paragraph, area);
    }

    fn render_parameters(&self, f: &mut Frame, area: Rect) {
        let config = &self.simulation.config;
        
        let text = Text::from(vec![
            Line::from(vec![
                Span::styled("Separation: ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:.1}", config.separation_weight), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Alignment: ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:.1}", config.alignment_weight), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Cohesion: ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:.1}", config.cohesion_weight), Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Max Speed: ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:.1}", config.max_speed), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Max Force: ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:.2}", config.max_force), Style::default().fg(Color::White)),
            ]),
        ]);

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title("Parameters")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
            );

        f.render_widget(paragraph, area);
    }
}