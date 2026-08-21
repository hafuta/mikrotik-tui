//go:build !windows

package credentials

import "os"

func replaceFile(source, destination string) error {
	return os.Rename(source, destination)
}
