package ui

import (
	"strings"

	"github.com/charmbracelet/lipgloss"
)

// BrailleSparkline renders a right-aligned single-series history.
type BrailleSparkline struct {
	Samples       []float64
	Width, Height int
	Min, Max      float64
	Style         lipgloss.Style
}

func (s BrailleSparkline) View() string {
	width, height := max(1, s.Width), max(1, s.Height)
	cells := make([][]byte, height)
	for row := range cells {
		cells[row] = make([]byte, width)
	}
	if len(s.Samples) == 0 {
		return strings.Repeat(strings.Repeat(" ", width)+"\n", height-1) + strings.Repeat(" ", width)
	}
	samples := s.Samples
	if len(samples) > width {
		samples = samples[len(samples)-width:]
	}
	minimum, maximum := s.Min, s.Max
	if maximum <= minimum {
		minimum, maximum = 0, 1
		for _, value := range samples {
			maximum = maxFloat(maximum, value)
		}
	}
	logicalWidth, logicalHeight := width*2, height*4
	startX := max(0, logicalWidth-1-(len(samples)-1)*2)
	point := func(index int) (int, int) {
		x := min(logicalWidth-1, startX+index*2)
		ratio := (samples[index] - minimum) / (maximum - minimum)
		y := logicalHeight - 1 - int(ratio*float64(logicalHeight-1))
		return x, clamp(y, 0, logicalHeight-1)
	}
	x, y := point(0)
	setSparkDot(cells, x, y)
	for index := 1; index < len(samples); index++ {
		nextX, nextY := point(index)
		drawSparkLine(cells, x, y, nextX, nextY)
		x, y = nextX, nextY
	}
	lines := make([]string, height)
	for row := range cells {
		var line strings.Builder
		for _, bits := range cells[row] {
			if bits == 0 {
				line.WriteByte(' ')
				continue
			}
			line.WriteRune(rune(0x2800) + rune(bits))
		}
		lines[row] = s.Style.Render(line.String())
	}
	return strings.Join(lines, "\n")
}

func drawSparkLine(cells [][]byte, x0, y0, x1, y1 int) {
	dx, dy := abs(x1-x0), -abs(y1-y0)
	stepX, stepY := -1, -1
	if x0 < x1 {
		stepX = 1
	}
	if y0 < y1 {
		stepY = 1
	}
	err := dx + dy
	for {
		setSparkDot(cells, x0, y0)
		if x0 == x1 && y0 == y1 {
			return
		}
		double := 2 * err
		if double >= dy {
			err += dy
			x0 += stepX
		}
		if double <= dx {
			err += dx
			y0 += stepY
		}
	}
}

func setSparkDot(cells [][]byte, x, y int) {
	if len(cells) == 0 || len(cells[0]) == 0 || x < 0 || y < 0 {
		return
	}
	cellX, cellY := x/2, y/4
	if cellY >= len(cells) || cellX >= len(cells[cellY]) {
		return
	}
	cells[cellY][cellX] |= brailleBit(x%2, y%4)
}
