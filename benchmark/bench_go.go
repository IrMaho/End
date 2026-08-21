// Benchmark 5: Go
package main

import "fmt"

type Request struct {
	ID          uint64
	PayloadSize int32
	Checksum    int64
}

func processRequest(id uint64, size int32) Request {
	var hash int64 = 17
	for j := int64(0); j < 50; j++ {
		hash = (hash * 31) + int64(id) + j
	}
	return Request{
		ID:          id,
		PayloadSize: size,
		Checksum:    hash,
	}
}

func main() {
	iterations := 1000000
	var totalChecksum int64 = 0

	fmt.Println("Running Go Backend Benchmark (1,000,000 requests)...")

	for i := 0; i < iterations; i++ {
		req := processRequest(uint64(i), 256)
		totalChecksum += req.Checksum
	}

	fmt.Printf("Go Benchmark Finished. Total Checksum:\n%d\n", totalChecksum)
}
