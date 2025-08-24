# Tamama - Terminal Boids Simulation

A real-time flocking simulation using the Boids algorithm, rendered in your terminal.

## Features

- **Dynamic Boid Count**: Automatically adjusts flock size based on terminal dimensions
- **Sine Wave Leadership**: Invisible leader guides the flock in smooth wave patterns
- **Adaptive Boundaries**: Boids bounce off terminal edges naturally
- **Live Controls**: Pause, reset, and adjust frame rate on the fly

## Installation

### From crates.io (Recommended)

```bash
cargo install tamama
```

### From GitHub Releases

Download the latest binary for your platform from the [releases page](https://github.com/yourusername/tamama/releases).

### From Source

```bash
git clone https://github.com/yourusername/tamama.git
cd tamama
cargo install --path .
```

## Usage

```bash
tamama
```

## Controls

- `Space` - Pause/Resume simulation
- `F` - Toggle between 30/60 FPS
- `R` - Reset simulation
- `Q` - Quit

## Requirements

- Rust 1.70+
- Terminal with Unicode support

---

Built with Rust 🦀 | Powered by [ratatui](https://github.com/ratatui-org/ratatui)