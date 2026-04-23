package model

const (
	FrontSlots = 4
	BackSlots  = 5
	TotalSlots = FrontSlots + BackSlots
)

func FrontSlot(index int) int {
	return index
}

func BackSlot(index int) int {
	return FrontSlots + index
}

func IsFront(slot int) bool {
	return slot >= 0 && slot < FrontSlots
}

func IsBack(slot int) bool {
	return slot >= FrontSlots && slot < TotalSlots
}

func SupportedFrontSlots(backSlot int) []int {
	switch backSlot {
	case 0:
		return []int{FrontSlot(0)}
	case 1:
		return []int{FrontSlot(0), FrontSlot(1)}
	case 2:
		return []int{FrontSlot(1), FrontSlot(2)}
	case 3:
		return []int{FrontSlot(2), FrontSlot(3)}
	case 4:
		return []int{FrontSlot(3)}
	default:
		return nil
	}
}

func SupportingBackSlots(frontSlot int) []int {
	switch frontSlot {
	case FrontSlot(0):
		return []int{BackSlot(0), BackSlot(1)}
	case FrontSlot(1):
		return []int{BackSlot(1), BackSlot(2)}
	case FrontSlot(2):
		return []int{BackSlot(2), BackSlot(3)}
	case FrontSlot(3):
		return []int{BackSlot(3), BackSlot(4)}
	default:
		return nil
	}
}
