package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"
	"time"
)

func xorshiftCompute(iterations uint64) uint64 {
	state := uint64(0x853c49e6748fea9b)
	for i := uint64(0); i < iterations; i++ {
		state ^= (state << 13)
		state ^= (state >> 7)
		state ^= (state << 17)
		state = state*6364136223846793005 + 1442695040888963407
	}
	return state
}

func healthHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Connection", "close")
	json.NewEncoder(w).Encode(map[string]string{
		"status": "ok",
		"lang":   "Go 1.25.1",
	})
}

func computeHandler(w http.ResponseWriter, r *http.Request) {
	nStr := r.URL.Query().Get("n")
	n := uint64(1000000)
	if nStr != "" {
		if parsed, err := strconv.ParseUint(nStr, 10, 64); err == nil {
			n = parsed
		}
	}
	t0 := time.Now()
	hash := xorshiftCompute(n)
	elapsed := time.Since(t0)
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Connection", "close")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"hash":    hash,
		"time_us": elapsed.Microseconds(),
		"lang":    "Go 1.25.1",
	})
}

func jsonHandler(w http.ResponseWriter, r *http.Request) {
	data := map[string]interface{}{
		"server":  "Go HTTP Backend",
		"version": "1.25.1",
		"users": []map[string]interface{}{
			{"id": 1, "name": "Alice", "score": 9850, "active": true},
			{"id": 2, "name": "Bob", "score": 8720, "active": true},
			{"id": 3, "name": "Charlie", "score": 7630, "active": false},
			{"id": 4, "name": "Diana", "score": 9210, "active": true},
			{"id": 5, "name": "Eve", "score": 8890, "active": true},
		},
		"metadata": map[string]interface{}{
			"total_users":   5,
			"avg_score":     8860,
			"active_count":  4,
			"server_uptime": 99.97,
		},
	}
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Connection", "close")
	json.NewEncoder(w).Encode(data)
}

func main() {
	http.HandleFunc("/health", healthHandler)
	http.HandleFunc("/compute", computeHandler)
	http.HandleFunc("/json", jsonHandler)
	fmt.Println("[Go] HTTP Server listening on :9004")
	http.ListenAndServe(":9004", nil)
}
