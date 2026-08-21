package main

import (
	"fmt"
	"time"
)

func benchCompute(iterations uint64) uint64 {
	state := uint64(0x853c49e6748fea9b)
	for i := uint64(0); i < iterations; i++ {
		state ^= (state << 13)
		state ^= (state >> 7)
		state ^= (state << 17)
		state = state*6364136223846793005 + 1442695040888963407
	}
	return state
}

func benchMemory(batches int, elementsPerBatch int) int64 {
	grandTotal := int64(0)
	for b := 0; b < batches; b++ {
		arr := make([]int64, elementsPerBatch)
		batchSum := int64(0)
		for i := 0; i < elementsPerBatch; i++ {
			val := int64(b*31 + i*17)
			arr[i] = val
			batchSum += val
		}
		grandTotal += (batchSum ^ arr[0])
	}
	return grandTotal
}

func fib(n int64) int64 {
	if n <= 1 {
		return n
	}
	return fib(n-1) + fib(n-2)
}

func main() {
	fmt.Println("=== Go Benchmark (go build -ldflags=\"-s -w\") ===")

	t0 := time.Now()
	res1 := benchCompute(100000000)
	t1 := time.Since(t0)
	fmt.Printf("1. Compute (100M iter): %.2f ms (Hash: %d)\n", float64(t1.Microseconds())/1000.0, res1)

	t0 = time.Now()
	res2 := benchMemory(5000, 20000)
	t1 = time.Since(t0)
	fmt.Printf("2. Memory Churn (100M items): %.2f ms (Sum: %d)\n", float64(t1.Microseconds())/1000.0, res2)

	t0 = time.Now()
	res3 := fib(42)
	t1 = time.Since(t0)
	fmt.Printf("3. Recursion (fib 42): %.2f ms (Val: %d)\n", float64(t1.Microseconds())/1000.0, res3)
}
