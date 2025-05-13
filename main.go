package main

import (
	"flag"
	"fmt"
	"math/rand"
	"os"
	"strings"
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

const (
	updateInterval        = 15 * time.Millisecond
	lightningChance       = 0.005
	lightningGrowthDelay  = 2 * time.Millisecond
	lightningMaxBranches  = 2
	lightningBranchChance = 0.3
	forkChance            = 0.15
	forkHorizontalSpread  = 3
	segmentLifespan       = 800 * time.Millisecond
)

var (
	rainChars      = []rune{'|', '.', '`'}
	lightningChars = []rune{'*', '+', '#'}
)

var lipglossColorMap = map[string]lipgloss.TerminalColor{
	"black":   lipgloss.Color("0"),
	"red":     lipgloss.Color("1"),
	"green":   lipgloss.Color("2"),
	"yellow":  lipgloss.Color("3"),
	"blue":    lipgloss.Color("4"),
	"magenta": lipgloss.Color("5"),
	"cyan":    lipgloss.Color("6"),
	"white":   lipgloss.Color("7"),
}

// Raindrop represents a single raindrop.
type Raindrop struct {
	x     int
	y     float64
	speed float64
	char  rune
}

// LightningSegment represents a segment of a lightning bolt.
type LightningSegment struct {
	y, x         int
	creationTime time.Time
}

// LightningBolt represents a lightning bolt.
type LightningBolt struct {
	segments       []LightningSegment
	targetLength   int
	lastGrowthTime time.Time
	isGrowing      bool
	maxY, maxX     int
	rng            *rand.Rand
}

// NewLightningBolt creates and initializes a new LightningBolt.
func NewLightningBolt(startRow, startCol, maxY, maxX int, rng *rand.Rand) *LightningBolt {
	minLength := maxY / 3
	if minLength < 1 {
		minLength = 1
	}
	maxLength := maxY - 2
	if maxLength <= minLength {
		maxLength = minLength + 1
	}
	if maxLength <= 0 {
		maxLength = 1
	}

	targetLen := rng.Intn(maxLength-minLength+1) + minLength
	if targetLen <= 0 {
		targetLen = 1
	}

	return &LightningBolt{
		segments:       []LightningSegment{{y: startRow, x: startCol, creationTime: time.Now()}},
		targetLength:   targetLen,
		lastGrowthTime: time.Now(),
		isGrowing:      true,
		maxY:           maxY,
		maxX:           maxX,
		rng:            rng,
	}
}

// grow attempts to grow the lightning bolt by adding new segments.
// It's called by UpdateBolt when conditions for growth are met.
func (lb *LightningBolt) grow(currentTime time.Time) {
	lb.lastGrowthTime = currentTime
	var newSegmentsThisStep []LightningSegment
	addedSegment := false

	if len(lb.segments) == 0 { // Should not happen if isGrowing is true, but as a safeguard
		lb.isGrowing = false
		return
	}

	lastSeg := lb.segments[len(lb.segments)-1]
	lastY, lastX := lastSeg.y, lastSeg.x

	if len(lb.segments) < lb.targetLength && lastY < lb.maxY-1 {
		branches := 1
		if lb.rng.Float64() < lightningBranchChance {
			branches = lb.rng.Intn(lightningMaxBranches) + 1
		}

		currentX := lastX
		nextPrimaryX := currentX // Store X of the first segment in this step for fork logic

		for i := 0; i < branches; i++ {
			offset := lb.rng.Intn(5) - 2 // -2 to +2
			nextX := currentX + offset
			if nextX < 0 {
				nextX = 0
			}
			if nextX >= lb.maxX {
				nextX = lb.maxX - 1
			}

			nextY := lastY + 1
			if nextY >= lb.maxY {
				nextY = lb.maxY - 1 // Clamp to maxY
			}

			newSegmentsThisStep = append(newSegmentsThisStep, LightningSegment{y: nextY, x: nextX, creationTime: currentTime})
			if i == 0 {
				nextPrimaryX = nextX
			}
			currentX = nextX // For multi-branch, subsequent branches fork from the previous one
			addedSegment = true
		}

		// Forking logic
		if lb.rng.Float64() < forkChance {
			forkOffset := lb.rng.Intn(2*forkHorizontalSpread+1) - forkHorizontalSpread
			if forkOffset == 0 { // Ensure fork is to a different column
				if lb.rng.Float64() < 0.5 {
					forkOffset = -1
				} else {
					forkOffset = 1
				}
			}
			forkX := lastX + forkOffset
			if forkX < 0 {
				forkX = 0
			}
			if forkX >= lb.maxX {
				forkX = lb.maxX - 1
			}

			forkY := lastY + 1 // Fork grows downwards
			if forkY >= lb.maxY {
				forkY = lb.maxY - 1
			}

			// Add fork only if it's different from the primary new segment's X
			if forkX != nextPrimaryX {
				newSegmentsThisStep = append(newSegmentsThisStep, LightningSegment{y: forkY, x: forkX, creationTime: currentTime})
				addedSegment = true
			}
		}
	}

	if !addedSegment || len(lb.segments) >= lb.targetLength || (len(lb.segments) > 0 && lb.segments[len(lb.segments)-1].y >= lb.maxY-1) {
		lb.isGrowing = false
	}

	if len(newSegmentsThisStep) > 0 {
		// Ensure unique new segments by position to avoid visual clutter
		uniqueNewMap := make(map[string]LightningSegment)
		for _, seg := range newSegmentsThisStep {
			key := fmt.Sprintf("%d,%d", seg.y, seg.x)
			uniqueNewMap[key] = seg
		}
		for _, seg := range uniqueNewMap {
			lb.segments = append(lb.segments, seg)
		}
	}
}

// UpdateBolt updates the state of the lightning bolt.
// It returns false if the bolt is no longer active (all segments expired and not growing).
// This method now uses a pointer receiver to modify the LightningBolt instance.
func (lb *LightningBolt) UpdateBolt() bool {
	currentTime := time.Now()

	if lb.isGrowing && currentTime.Sub(lb.lastGrowthTime) >= lightningGrowthDelay {
		lb.grow(currentTime)
	}

	if len(lb.segments) == 0 && !lb.isGrowing {
		return false // Bolt is gone and won't grow
	}

	// Check if all segments have expired
	allExpired := true
	if len(lb.segments) > 0 {
		for _, seg := range lb.segments {
			if currentTime.Sub(seg.creationTime) <= segmentLifespan {
				allExpired = false // Found an active segment
				break
			}
		}
	} else if lb.isGrowing { // If no segments yet but set to grow (e.g. initial state)
		allExpired = false
	}

	return !allExpired // Active if not all segments are expired or if it's still set to grow
}

// tickMsg is a message sent on every tick to update the animation.
type tickMsg time.Time

// model represents the state of the TUI application.
type model struct {
	width, height  int
	raindrops      []*Raindrop
	activeBolts    []*LightningBolt
	isThunderstorm bool
	rainStyle      lipgloss.Style
	lightningStyle lipgloss.Style
	rng            *rand.Rand
	quitting       bool
	screenBuffer   [][]CellData
	defaultCell    CellData
}

// CellData stores character and style for a single cell on the screen.
type CellData struct {
	char  rune
	style lipgloss.Style
}

// initialModel creates the initial state of the application model.
func initialModel(rainColorName, lightningColorName string) model {
	rc, ok := lipglossColorMap[strings.ToLower(rainColorName)]
	if !ok {
		rc = lipglossColorMap["cyan"] // Default rain color
	}
	lc, ok := lipglossColorMap[strings.ToLower(lightningColorName)]
	if !ok {
		lc = lipglossColorMap["yellow"] // Default lightning color
	}

	s := rand.NewSource(time.Now().UnixNano())
	r := rand.New(s)

	return model{
		rainStyle:      lipgloss.NewStyle().Foreground(rc),
		lightningStyle: lipgloss.NewStyle().Foreground(lc).Bold(true),
		rng:            r,
		defaultCell:    CellData{char: ' ', style: lipgloss.NewStyle()},
	}
}

// Init is called once when the program starts.
func (m model) Init() tea.Cmd {
	return tickCmd()
}

// tickCmd creates a command that sends a tickMsg after updateInterval.
func tickCmd() tea.Cmd {
	return tea.Tick(updateInterval, func(t time.Time) tea.Msg {
		return tickMsg(t)
	})
}

// updateLightningSystem handles the logic for creating and updating lightning bolts.
func (m model) updateLightningSystem() model {
	if !m.isThunderstorm {
		// Clear existing bolts if thunderstorm mode is turned off
		if len(m.activeBolts) > 0 {
			m.activeBolts = nil
		}
	} else {
		// Attempt to create a new bolt
		if len(m.activeBolts) < 3 && m.rng.Float64() < lightningChance {
			startC := 0
			if m.width > 0 {
				startC = m.rng.Intn(m.width/2) + m.width/4 // Spawn in middle half
			}

			startR := 0
			if m.height/5 > 0 {
				startR = m.rng.Intn(m.height / 5) // Spawn near the top
			}

			if m.height > 0 && m.width > 0 { // Ensure valid dimensions
				m.activeBolts = append(m.activeBolts, NewLightningBolt(startR, startC, m.height, m.width, m.rng))
			}
		}
	}

	// Update existing bolts
	var nextBolts []*LightningBolt
	for _, bolt := range m.activeBolts {
		if bolt.UpdateBolt() { // UpdateBolt now correctly modifies bolt due to pointer receiver
			nextBolts = append(nextBolts, bolt)
		}
	}
	m.activeBolts = nextBolts
	return m
}

// updateRaindropSystem handles the logic for creating and updating raindrops.
func (m model) updateRaindropSystem() model {
	generationChance := 0.3
	maxNewDrops := m.width / 15
	minSpeed := 0.1
	maxSpeed := 0.6

	if m.isThunderstorm {
		generationChance = 0.5
		maxNewDrops = m.width / 8
		maxSpeed = 1.0
	}
	if maxNewDrops < 1 {
		maxNewDrops = 1
	}

	// Generate new raindrops
	if m.rng.Float64() < generationChance {
		numNewDrops := 0
		if maxNewDrops > 1 {
			numNewDrops = m.rng.Intn(maxNewDrops) + 1
		} else if maxNewDrops == 1 {
			numNewDrops = 1
		}

		for range numNewDrops {
			x := 0
			if m.width > 0 {
				x = m.rng.Intn(m.width)
			}
			speed := m.rng.Float64()*(maxSpeed-minSpeed) + minSpeed
			char := rainChars[m.rng.Intn(len(rainChars))]
			m.raindrops = append(m.raindrops, &Raindrop{x: x, y: 0.0, speed: speed, char: char})
		}
	}

	// Update existing raindrops
	var nextRaindrops []*Raindrop
	for _, drop := range m.raindrops {
		drop.y += drop.speed
		if int(drop.y) < m.height {
			nextRaindrops = append(nextRaindrops, drop)
		}
	}
	m.raindrops = nextRaindrops
	return m
}

// Update handles messages and updates the model.
func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	if m.quitting {
		return m, tea.Quit
	}

	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "q", "ctrl+c", "esc":
			m.quitting = true
			return m, tea.Quit
		case "t":
			m.isThunderstorm = !m.isThunderstorm
			// When toggling, clear existing effects for a cleaner transition
			m.raindrops = nil
			m.activeBolts = nil
			return m, nil // No command needed, just state change
		}

	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height
		// Clear effects on resize to prevent out-of-bounds issues
		m.raindrops = nil
		m.activeBolts = nil

		if m.width > 0 && m.height > 0 {
			newScreenBuffer := make([][]CellData, m.height)
			for i := range newScreenBuffer {
				newScreenBuffer[i] = make([]CellData, m.width)
				for j := range newScreenBuffer[i] {
					newScreenBuffer[i][j] = m.defaultCell
				}
			}
			m.screenBuffer = newScreenBuffer
		} else {
			m.screenBuffer = nil
		}
		return m, nil // No command needed

	case tickMsg:
		if m.width <= 0 || m.height <= 0 { // Avoid updates if dimensions are invalid
			return m, tickCmd()
		}

		m = m.updateLightningSystem()
		m = m.updateRaindropSystem()

		return m, tickCmd()
	}
	return m, nil
}

// View renders the current state of the model as a string.
func (m model) View() string {
	if m.quitting || m.width <= 0 || m.height <= 0 || m.screenBuffer == nil {
		return ""
	}

	for i := range m.screenBuffer {
		for j := range m.screenBuffer[i] {
			m.screenBuffer[i][j] = m.defaultCell
		}
	}

	// Draw raindrops
	currentRainStyle := m.rainStyle
	if m.isThunderstorm {
		currentRainStyle = m.rainStyle.Bold(true)
	}
	for _, drop := range m.raindrops {
		yPos := int(drop.y)
		xPos := drop.x
		if yPos >= 0 && yPos < m.height && xPos >= 0 && xPos < m.width {
			m.screenBuffer[yPos][xPos] = CellData{char: drop.char, style: currentRainStyle}
		}
	}

	// Draw lightning
	currentTime := time.Now()
	maxCharIndex := len(lightningChars) - 1
	for _, bolt := range m.activeBolts {
		for _, seg := range bolt.segments {
			if seg.y >= 0 && seg.y < m.height && seg.x >= 0 && seg.x < m.width {
				segmentAge := currentTime.Sub(seg.creationTime)

				if segmentAge <= segmentLifespan {
					normAge := float64(segmentAge) / float64(segmentLifespan) // Normalized age [0, 1]

					// Determine character based on age (brighter when newer)
					var charIndex int
					if normAge < 0.33 { // Newest
						charIndex = 2
					} else if normAge < 0.66 { // Middle age
						charIndex = 1
					} else { // Oldest
						charIndex = 0
					}
					// Clamp charIndex to be within bounds of lightningChars
					charIndex = max(0, min(maxCharIndex, charIndex))
					m.screenBuffer[seg.y][seg.x] = CellData{char: lightningChars[charIndex], style: m.lightningStyle}
				}
			}
		}
	}

	// Build the final string for display
	var b strings.Builder
	b.Grow(m.height * (m.width + 1)) // Pre-allocate for efficiency
	for r := range m.height {
		for c := range m.width {
			cell := m.screenBuffer[r][c]
			b.WriteString(cell.style.Render(string(cell.char)))
		}
		if r < m.height-1 { // Add newline for all but the last line
			b.WriteRune('\n')
		}
	}
	return b.String()
}

func main() {
	defaultRainColor := "cyan"
	defaultLightningColor := "yellow"

	rainColorArg := flag.String("rain-color", defaultRainColor, "Color for the rain. Choices: black, red, green, yellow, blue, magenta, cyan, white")
	lightningColorArg := flag.String("lightning-color", defaultLightningColor, "Color for the lightning. Choices: black, red, green, yellow, blue, magenta, cyan, white")

	flag.Parse()

	// Validate color inputs
	if _, ok := lipglossColorMap[strings.ToLower(*rainColorArg)]; !ok {
		fmt.Printf("Warning: Invalid rain color '%s'. Using default '%s'.\n", *rainColorArg, defaultRainColor)
		*rainColorArg = defaultRainColor
	}
	if _, ok := lipglossColorMap[strings.ToLower(*lightningColorArg)]; !ok {
		fmt.Printf("Warning: Invalid lightning color '%s'. Using default '%s'.\n", *lightningColorArg, defaultLightningColor)
		*lightningColorArg = defaultLightningColor
	}

	m := initialModel(*rainColorArg, *lightningColorArg)
	p := tea.NewProgram(m, tea.WithAltScreen())

	// Initial messages to the user
	fmt.Println("Initializing Terminal Weather Simulation (Rain/Lightning)...")
	fmt.Printf("Configuration loaded: Rain color=%s, Lightning color=%s\n", *rainColorArg, *lightningColorArg)
	fmt.Println("Controls: 't' - Toggle thunderstorm mode, 'q'|ESC|Ctrl+C - Exit program.")
	time.Sleep(1 * time.Second)

	if _, err := p.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "Error running program: %v\n", err)
		os.Exit(1)
	}
	fmt.Println("\nTerminal weather simulation process terminated.")
}
