# Tamama

A terminal-based weather animation system that simulates rain and lightning in your console.

## Features

- Realistic rain animation with variable intensity
- Dynamic lightning effects during thunderstorms
- Wind effects that change the direction of rainfall
- Customizable colors for both rain and lightning
- Interactive controls to adjust the weather simulation

## Installation

```
go install github.com/omegaatt36/tamama@latest
```

Or clone the repository and build from source:

```
git clone https://github.com/omegaatt36/tamama.git
cd tamama
go build
```

## Usage

Run the program with default settings:

```
tamama
```

### Command-line Options

- `--rain-color`: Set the color for rain (black, red, green, yellow, blue, magenta, cyan, white)
- `--lightning-color`: Set the color for lightning (black, red, green, yellow, blue, magenta, cyan, white)

Example:

```
tamama --rain-color blue --lightning-color yellow
```

### Interactive Controls

- `t`: Toggle thunderstorm mode
- `←/→`: Adjust wind direction (max ±30°)
- `q`, `ESC`, `Ctrl+C`: Exit the program

## Requirements

- Go 1.24.0 or higher
- Terminal with ANSI color support

## Dependencies

- [Bubble Tea](https://github.com/charmbracelet/bubbletea): Terminal UI framework
- [Lipgloss](https://github.com/charmbracelet/lipgloss): Style definitions for terminal applications

## License

This project is open source and available under the [MIT License](LICENSE).