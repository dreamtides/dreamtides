package model

import (
	"reflect"
	"testing"
)

func TestSupportedFrontSlots(t *testing.T) {
	tests := []struct {
		name     string
		backSlot int
		want     []int
	}{
		{name: "left edge", backSlot: 0, want: []int{FrontSlot(0)}},
		{name: "left interior", backSlot: 1, want: []int{FrontSlot(0), FrontSlot(1)}},
		{name: "center", backSlot: 2, want: []int{FrontSlot(1), FrontSlot(2)}},
		{name: "right interior", backSlot: 3, want: []int{FrontSlot(2), FrontSlot(3)}},
		{name: "right edge", backSlot: 4, want: []int{FrontSlot(3)}},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			got := SupportedFrontSlots(test.backSlot)
			if !reflect.DeepEqual(got, test.want) {
				t.Fatalf("SupportedFrontSlots(%d) = %v, want %v", test.backSlot, got, test.want)
			}
		})
	}
}

func TestSupportingBackSlots(t *testing.T) {
	tests := []struct {
		name      string
		frontSlot int
		want      []int
	}{
		{name: "left front", frontSlot: FrontSlot(0), want: []int{BackSlot(0), BackSlot(1)}},
		{name: "left center front", frontSlot: FrontSlot(1), want: []int{BackSlot(1), BackSlot(2)}},
		{name: "right center front", frontSlot: FrontSlot(2), want: []int{BackSlot(2), BackSlot(3)}},
		{name: "right front", frontSlot: FrontSlot(3), want: []int{BackSlot(3), BackSlot(4)}},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			got := SupportingBackSlots(test.frontSlot)
			if !reflect.DeepEqual(got, test.want) {
				t.Fatalf("SupportingBackSlots(%d) = %v, want %v", test.frontSlot, got, test.want)
			}
		})
	}
}
