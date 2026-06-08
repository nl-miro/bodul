package main

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"unicode"
)

var checkMode bool

func main() {
	flag.BoolVar(&checkMode, "check", false, "Check mode: exit non-zero if changes needed (CI-friendly)")
	flag.Usage = func() {
		fmt.Fprintf(os.Stderr, "Usage: %s [flags] <directory>\n", os.Args[0])
		flag.PrintDefaults()
	}
	flag.Parse()

	if flag.NArg() < 1 {
		flag.Usage()
		os.Exit(1)
	}

	dir := flag.Arg(0)
	info, err := os.Stat(dir)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
	if !info.IsDir() {
		fmt.Fprintf(os.Stderr, "Error: %s is not a directory\n", dir)
		os.Exit(1)
	}

	needsChanges := false
	err = filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if !info.IsDir() && strings.HasSuffix(strings.ToLower(path), ".md") {
			changed, err := processFile(path)
			if err != nil {
				fmt.Fprintf(os.Stderr, "Warning: failed to process %s: %v\n", path, err)
			}
			if changed {
				needsChanges = true
			}
		}
		return nil
	})

	if err != nil {
		fmt.Fprintf(os.Stderr, "Error walking directory: %v\n", err)
		os.Exit(1)
	}

	if checkMode && needsChanges {
		os.Exit(1)
	}
}

func processFile(path string) (bool, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return false, err
	}

	lines := strings.Split(string(content), "\n")
	modified := false
	result := make([]string, 0, len(lines))

	i := 0
	for i < len(lines) {
		if isTableRow(lines[i]) {
			// Collect all consecutive table rows
			tableStart := i
			for i < len(lines) && isTableRow(lines[i]) {
				i++
			}
			tableLines := lines[tableStart:i]

			// Align the table
			aligned := alignTable(tableLines)

			// Check if anything changed
			for j, line := range aligned {
				if line != tableLines[j] {
					modified = true
					break
				}
			}

			result = append(result, aligned...)
		} else {
			result = append(result, lines[i])
			i++
		}
	}

	if modified {
		if checkMode {
			fmt.Printf("Needs alignment: %s\n", path)
		} else {
			output := strings.Join(result, "\n")
			if err := os.WriteFile(path, []byte(output), 0644); err != nil {
				return false, err
			}
			fmt.Printf("Aligned: %s\n", path)
		}
	}

	return modified, nil
}

func isTableRow(line string) bool {
	trimmed := strings.TrimSpace(line)
	return strings.HasPrefix(trimmed, "|") && strings.HasSuffix(trimmed, "|")
}

func alignTable(lines []string) []string {
	if len(lines) == 0 {
		return lines
	}

	// Parse all rows into cells
	rows := make([][]string, len(lines))
	maxCols := 0

	for i, line := range lines {
		cells := parseCells(line)
		rows[i] = cells
		if len(cells) > maxCols {
			maxCols = len(cells)
		}
	}

	// Normalize row lengths
	for i := range rows {
		for len(rows[i]) < maxCols {
			rows[i] = append(rows[i], "")
		}
	}

	// Calculate max width for each column (exclude separator rows)
	colWidths := make([]int, maxCols)
	for i, row := range rows {
		if isSeparatorRow(lines[i]) {
			continue
		}
		for j, cell := range row {
			width := cellWidth(cell)
			if width > colWidths[j] {
				colWidths[j] = width
			}
		}
	}

	// Ensure minimum width of 3 for separator rows
	for i := range colWidths {
		if colWidths[i] < 3 {
			colWidths[i] = 3
		}
	}

	// Rebuild rows with aligned cells
	result := make([]string, len(rows))
	for i, row := range rows {
		if isSeparatorRow(lines[i]) {
			result[i] = buildSeparatorRow(lines[i], row, colWidths)
		} else {
			result[i] = buildDataRow(row, colWidths)
		}
	}

	return result
}

func parseCells(line string) []string {
	trimmed := strings.TrimSpace(line)
	// Remove leading and trailing pipes
	trimmed = strings.TrimPrefix(trimmed, "|")
	trimmed = strings.TrimSuffix(trimmed, "|")

	parts := strings.Split(trimmed, "|")
	cells := make([]string, len(parts))
	for i, part := range parts {
		cells[i] = strings.TrimSpace(part)
	}
	return cells
}

func cellWidth(cell string) int {
	width := 0
	lastRuneWidth := 0
	for _, r := range cell {
		if r == 0xfe0f {
			if lastRuneWidth == 1 {
				width++
				lastRuneWidth = 2
			}
			continue
		}

		lastRuneWidth = runeWidth(r)
		width += lastRuneWidth
	}
	return width
}

func runeWidth(r rune) int {
	if r == 0 {
		return 0
	}
	if r < 32 || (r >= 0x7f && r < 0xa0) {
		return 0
	}
	if unicode.Is(unicode.Mn, r) || unicode.Is(unicode.Me, r) || unicode.Is(unicode.Cf, r) {
		return 0
	}
	if isWideRune(r) {
		return 2
	}
	return 1
}

func isWideRune(r rune) bool {
	return inRanges(r, [][2]rune{
		{0x1100, 0x115f},
		{0x231a, 0x231b},
		{0x2329, 0x232a},
		{0x23e9, 0x23ec},
		{0x23f0, 0x23f0},
		{0x23f3, 0x23f3},
		{0x25fd, 0x25fe},
		{0x2614, 0x2615},
		{0x2648, 0x2653},
		{0x267f, 0x267f},
		{0x2693, 0x2693},
		{0x26a1, 0x26a1},
		{0x26aa, 0x26ab},
		{0x26bd, 0x26be},
		{0x26c4, 0x26c5},
		{0x26ce, 0x26ce},
		{0x26d4, 0x26d4},
		{0x26ea, 0x26ea},
		{0x26f2, 0x26f3},
		{0x26f5, 0x26f5},
		{0x26fa, 0x26fa},
		{0x26fd, 0x26fd},
		{0x2705, 0x2705},
		{0x270a, 0x270b},
		{0x2728, 0x2728},
		{0x274c, 0x274c},
		{0x274e, 0x274e},
		{0x2753, 0x2755},
		{0x2757, 0x2757},
		{0x2795, 0x2797},
		{0x27b0, 0x27b0},
		{0x27bf, 0x27bf},
		{0x2b1b, 0x2b1c},
		{0x2b50, 0x2b50},
		{0x2b55, 0x2b55},
		{0x2e80, 0xa4cf},
		{0xac00, 0xd7a3},
		{0xf900, 0xfaff},
		{0xfe10, 0xfe19},
		{0xfe30, 0xfe6f},
		{0xff00, 0xff60},
		{0xffe0, 0xffe6},
		{0x1f000, 0x1faff},
		{0x20000, 0x3fffd},
	})
}

func inRanges(r rune, ranges [][2]rune) bool {
	for _, bounds := range ranges {
		if r >= bounds[0] && r <= bounds[1] {
			return true
		}
	}
	return false
}

func isSeparatorRow(line string) bool {
	trimmed := strings.TrimSpace(line)
	trimmed = strings.TrimPrefix(trimmed, "|")
	trimmed = strings.TrimSuffix(trimmed, "|")

	for _, part := range strings.Split(trimmed, "|") {
		part = strings.TrimSpace(part)
		if part == "" {
			continue
		}
		// Check if it's a separator pattern: dashes with optional colons
		cleaned := strings.Trim(part, ":-")
		if cleaned != "" {
			return false
		}
	}
	return true
}

func buildSeparatorRow(originalLine string, cells []string, widths []int) string {
	var parts []string
	originalCells := parseCells(originalLine)

	for i, width := range widths {
		// Preserve alignment markers from original
		original := ""
		if i < len(originalCells) {
			original = originalCells[i]
		}

		leftColon := strings.HasPrefix(original, ":")
		rightColon := strings.HasSuffix(original, ":")

		dashCount := width
		if leftColon {
			dashCount--
		}
		if rightColon {
			dashCount--
		}
		if dashCount < 1 {
			dashCount = 1
		}

		cell := strings.Repeat("-", dashCount)
		if leftColon {
			cell = ":" + cell
		}
		if rightColon {
			cell = cell + ":"
		}

		parts = append(parts, " "+cell+" ")
	}

	return "|" + strings.Join(parts, "|") + "|"
}

func buildDataRow(cells []string, widths []int) string {
	var parts []string
	for i, cell := range cells {
		width := widths[i]
		padded := cell + strings.Repeat(" ", width-cellWidth(cell))
		parts = append(parts, " "+padded+" ")
	}
	return "|" + strings.Join(parts, "|") + "|"
}
