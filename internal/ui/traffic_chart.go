package ui

import (
	"fmt"
	"strings"
	"time"

	"github.com/hafuta/mikrotik-tui/internal/theme"
)

// TrafficSample contains receive and transmit rates in bits per second.
type TrafficSample struct {
	RX float64
	TX float64
}

// TrafficChart renders two live series on a compact Braille canvas.
type TrafficChart struct {
	Samples        []TrafficSample
	Width, Height  int
	SampleInterval time.Duration
}

type trafficCell struct {
	rx, tx byte
}

func (c TrafficChart) View() string {
	width, height := max(1, c.Width), max(1, c.Height)
	labelWidth := min(10, max(5, width/3))
	plotWidth := max(1, width-labelWidth-3)
	plotHeight := max(1, height-1)
	samples := c.Samples
	if len(samples) > plotWidth {
		samples = samples[len(samples)-plotWidth:]
	}
	peak := 1.0
	for _, sample := range samples {
		peak = maxFloat(peak, sample.RX, sample.TX)
	}
	peak *= 1.08
	peakLabel := formatTrafficRate(peak)
	if len(c.Samples) == 0 {
		peakLabel = "—"
	}

	logicalWidth, logicalHeight := plotWidth*2, plotHeight*4
	cells := make([][]trafficCell, plotHeight)
	for row := range cells {
		cells[row] = make([]trafficCell, plotWidth)
	}

	plotSeries(cells, samples, logicalWidth, logicalHeight, peak, true)
	plotSeries(cells, samples, logicalWidth, logicalHeight, peak, false)

	lines := make([]string, 0, height)
	for row := range cells {
		var line strings.Builder
		for _, cell := range cells[row] {
			bits := cell.rx | cell.tx
			character := " "
			if bits != 0 {
				character = string(rune(0x2800) + rune(bits))
			}
			switch {
			case cell.rx != 0 && cell.tx != 0:
				line.WriteString(theme.Default.Alert.Render(character))
			case cell.rx != 0:
				line.WriteString(theme.Default.Signal.Render(character))
			case cell.tx != 0:
				line.WriteString(theme.Default.Focus.Render(character))
			default:
				line.WriteByte(' ')
			}
		}
		label := ""
		if row == 0 {
			label = peakLabel
		} else if row == len(cells)-1 {
			label = "0 bps"
		}
		lines = append(lines, theme.Default.Muted.Render(fitCell(label, labelWidth))+" "+
			theme.Default.Muted.Render("│")+" "+line.String())
	}
	interval := c.SampleInterval
	if interval <= 0 {
		interval = 2 * time.Second
	}
	window := interval * time.Duration(max(0, plotWidth-1))
	axis := trafficTimeAxis(plotWidth, window)
	lines = append(lines, strings.Repeat(" ", labelWidth+1)+theme.Default.Muted.Render("└ "+axis))
	return strings.Join(lines, "\n")
}

func trafficTimeAxis(width int, window time.Duration) string {
	if width <= 0 {
		return ""
	}
	left := "-" + formatTrafficWindow(window)
	right := "now"
	if width <= len(left)+len(right)+1 {
		return fitCell(right, width)
	}
	return left + strings.Repeat("─", width-len(left)-len(right)) + right
}

func formatTrafficWindow(window time.Duration) string {
	if window < time.Minute {
		return fmt.Sprintf("%ds", int(window.Seconds()))
	}
	minutes := int(window.Minutes())
	seconds := int(window.Seconds()) % 60
	if seconds == 0 {
		return fmt.Sprintf("%dm", minutes)
	}
	return fmt.Sprintf("%dm%02ds", minutes, seconds)
}

func formatTrafficRate(bitsPerSecond float64) string {
	switch {
	case bitsPerSecond >= 1_000_000_000:
		return fmt.Sprintf("%.1f Gbps", bitsPerSecond/1_000_000_000)
	case bitsPerSecond >= 1_000_000:
		return fmt.Sprintf("%.1f Mbps", bitsPerSecond/1_000_000)
	case bitsPerSecond >= 1_000:
		return fmt.Sprintf("%.1f Kbps", bitsPerSecond/1_000)
	default:
		return fmt.Sprintf("%.0f bps", bitsPerSecond)
	}
}

func plotSeries(cells [][]trafficCell, samples []TrafficSample, width, height int, peak float64, receive bool) {
	if len(samples) == 0 {
		return
	}
	valueAt := func(index int) float64 {
		if receive {
			return samples[index].RX
		}
		return samples[index].TX
	}
	startX := max(0, width-1-(len(samples)-1)*2)
	point := func(index int) (int, int) {
		x := min(width-1, startX+index*2)
		value := valueAt(index)
		y := height - 1 - int((value/peak)*float64(height-1))
		return x, clamp(y, 0, height-1)
	}

	previousX, previousY := point(0)
	setTrafficDot(cells, previousX, previousY, receive)
	for index := 1; index < len(samples); index++ {
		currentX, currentY := point(index)
		drawTrafficLine(cells, previousX, previousY, currentX, currentY, receive)
		previousX, previousY = currentX, currentY
	}
}

func drawTrafficLine(cells [][]trafficCell, x0, y0, x1, y1 int, receive bool) {
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
		setTrafficDot(cells, x0, y0, receive)
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

func setTrafficDot(cells [][]trafficCell, x, y int, receive bool) {
	if len(cells) == 0 || len(cells[0]) == 0 || x < 0 || y < 0 {
		return
	}
	cellX, cellY := x/2, y/4
	if cellY >= len(cells) || cellX >= len(cells[cellY]) {
		return
	}
	bit := brailleBit(x%2, y%4)
	if receive {
		cells[cellY][cellX].rx |= bit
	} else {
		cells[cellY][cellX].tx |= bit
	}
}

func brailleBit(x, y int) byte {
	bits := [4][2]byte{
		{0x01, 0x08},
		{0x02, 0x10},
		{0x04, 0x20},
		{0x40, 0x80},
	}
	return bits[y][x]
}

func abs(value int) int {
	if value < 0 {
		return -value
	}
	return value
}

func maxFloat(values ...float64) float64 {
	result := values[0]
	for _, value := range values[1:] {
		if value > result {
			result = value
		}
	}
	return result
}
