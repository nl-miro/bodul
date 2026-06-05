package main

import (
	"strings"
	"testing"
)

func TestCellWidthUsesDisplayColumns(t *testing.T) {
	tests := []struct {
		name string
		cell string
		want int
	}{
		{name: "ascii", cell: "abc", want: 3},
		{name: "text mark", cell: "✗", want: 1},
		{name: "check emoji", cell: "✅", want: 2},
		{name: "variation selector is zero width", cell: "✔️", want: 2},
		{name: "cjk", cell: "漢", want: 2},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := cellWidth(tt.cell); got != tt.want {
				t.Fatalf("cellWidth(%q) = %d, want %d", tt.cell, got, tt.want)
			}
		})
	}
}

func TestAlignTablePadsWideEmojiByDisplayWidth(t *testing.T) {
	lines := []string{
		"| Draft | 1 Intake | 2 Disc |",
		"|-------|----------|--------|",
		"| a | ✅ | ◐ |",
		"| b | ◐ | ✅ |",
		"| c | ✗ | ✗ |",
	}

	got := alignTable(lines)
	want := []string{
		"| Draft | 1 Intake | 2 Disc |",
		"|-------|----------|--------|",
		"| a     | ✅       | ◐      |",
		"| b     | ◐        | ✅     |",
		"| c     | ✗        | ✗      |",
	}

	if strings.Join(got, "\n") != strings.Join(want, "\n") {
		t.Fatalf("alignTable() =\n%s\nwant\n%s", strings.Join(got, "\n"), strings.Join(want, "\n"))
	}
}
