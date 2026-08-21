// Ultra-Heavy 10M Benchmark: Go (Garbage Collected Runtime)
package main

import (
	"fmt"
)

type Request struct {
	ID          uint64
	PayloadSize int32
	Checksum    int64
}

func processRequest(id uint64, size int32) Request {
	var hash int64 = 17
	for j := int64(0); j < 32; j++ {
		hash = (hash * 31) + int64(id) + j
	}
	return Request{
		ID:          id,
		PayloadSize: size,
		Checksum:    hash,
	}
}

func main() {
	const iterations = 10000000
	var totalChecksum int64 = 0

	fmt.Println("Running Go 10,000,000 Heavy Backend Requests Benchmark...")

	for i := uint64(0); i < iterations; i++ {
		req := processRequest(i, 256)
		totalChecksum += req.Checksum
	}

	fmt.Println("Go 10M Benchmark Finished. Total Checksum:")
	fmt.Println(totalChecksum)
}
