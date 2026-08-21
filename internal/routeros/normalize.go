package routeros

import (
	"fmt"
	"strconv"
	"strings"
)

// ParseBool accepts RouterOS boolean spellings while rejecting ambiguous data.
func ParseBool(value string) (bool, error) {
	switch strings.ToLower(strings.TrimSpace(value)) {
	case "true", "yes", "on", "1":
		return true, nil
	case "false", "no", "off", "0":
		return false, nil
	default:
		return false, fmt.Errorf("invalid RouterOS boolean %q", value)
	}
}

// ParseInt parses a base-10 RouterOS integer.
func ParseInt(value string) (int64, error) {
	normalized := strings.TrimSpace(value)
	result, err := strconv.ParseInt(normalized, 10, 64)
	if err != nil {
		return 0, fmt.Errorf("invalid RouterOS integer %q: %w", value, err)
	}
	return result, nil
}
