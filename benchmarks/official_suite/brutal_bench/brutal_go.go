package main

import (
	"fmt"
	"math"
	"time"
)

// ============================================================================
// 1. Raymarcher
// ============================================================================
type Vec3 struct{ x, y, z float64 }

func v3(x, y, z float64) Vec3 { return Vec3{x, y, z} }
func vAdd(a, b Vec3) Vec3     { return Vec3{a.x + b.x, a.y + b.y, a.z + b.z} }
func vSub(a, b Vec3) Vec3     { return Vec3{a.x - b.x, a.y - b.y, a.z - b.z} }
func vMul(a Vec3, s float64) Vec3 { return Vec3{a.x * s, a.y * s, a.z * s} }
func vDot(a, b Vec3) float64  { return a.x*b.x + a.y*b.y + a.z*b.z }
func vLen(a Vec3) float64     { return math.Sqrt(vDot(a, a)) }
func vNorm(a Vec3) Vec3 {
	l := vLen(a)
	if l > 1e-9 {
		return vMul(a, 1.0/l)
	}
	return a
}

func sdfSphere(p, c Vec3, r float64) float64 {
	return vLen(vSub(p, c)) - r
}
func sdfTorus(p Vec3, tx, ty float64) float64 {
	qx := math.Sqrt(p.x*p.x+p.z*p.z) - tx
	return math.Sqrt(qx*qx+p.y*p.y) - ty
}
func sdfScene(p Vec3) float64 {
	d1 := sdfSphere(p, v3(0.0, 0.0, 3.0), 0.8)
	d2 := sdfTorus(vSub(p, v3(0.0, -0.2, 3.0)), 1.2, 0.25)
	d3 := p.y + 1.2
	d := d1
	if d2 < d {
		d = d2
	}
	if d3 < d {
		d = d3
	}
	return d
}
func calcNormal(p Vec3) Vec3 {
	eps := 0.001
	d := sdfScene(p)
	n := v3(
		sdfScene(v3(p.x+eps, p.y, p.z))-d,
		sdfScene(v3(p.x, p.y+eps, p.z))-d,
		sdfScene(v3(p.x, p.y, p.z+eps))-d,
	)
	return vNorm(n)
}

func benchRaymarch(width, height int) uint64 {
	ro := v3(0.0, 0.5, -1.5)
	lightPos := v3(2.0, 4.0, -1.0)
	var totalLum uint64 = 0

	for y := 0; y < height; y++ {
		ny := (float64(y)/float64(height))*2.0 - 1.0
		for x := 0; x < width; x++ {
			nx := (float64(x)/float64(width))*2.0 - 1.0
			rd := vNorm(v3(nx*1.2, -ny, 1.5))

			t := 0.0
			hit := 0.0
			for step := 0; step < 64; step++ {
				p := vAdd(ro, vMul(rd, t))
				d := sdfScene(p)
				if d < 0.001 {
					n := calcNormal(p)
					ld := vNorm(vSub(lightPos, p))
					diff := vDot(n, ld)
					if diff < 0.0 {
						diff = 0.0
					}
					hit = diff * 255.0
					break
				}
				t += d
				if t > 20.0 {
					break
				}
			}
			totalLum += uint64(hit)
		}
	}
	return totalLum
}

// ============================================================================
// 2. Binary Trees
// ============================================================================
type TreeNode struct {
	item        int32
	left, right *TreeNode
}

func createTree(item int32, depth int32) *TreeNode {
	if depth > 0 {
		return &TreeNode{
			item:  item,
			left:  createTree(2*item-1, depth-1),
			right: createTree(2*item, depth-1),
		}
	}
	return &TreeNode{item: item}
}

func checkTree(n *TreeNode) int64 {
	if n == nil {
		return 0
	}
	sum := int64(n.item)
	if n.left != nil {
		sum += checkTree(n.left) - checkTree(n.right)
	}
	return sum
}

func benchBinaryTrees(maxDepth int32) int64 {
	minDepth := int32(4)
	var grandSum int64 = 0

	stretch := createTree(0, maxDepth+1)
	grandSum += checkTree(stretch)

	longLived := createTree(0, maxDepth)

	for depth := minDepth; depth <= maxDepth; depth += 2 {
		iterations := int32(1 << (maxDepth - depth + minDepth))
		var check int64 = 0
		for i := int32(1); i <= iterations; i++ {
			t1 := createTree(i, depth)
			check += checkTree(t1)

			t2 := createTree(-i, depth)
			check += checkTree(t2)
		}
		grandSum += check
	}

	grandSum += checkTree(longLived)
	return grandSum
}

// ============================================================================
// 3. HFT Engine
// ============================================================================
const maxLevels = 100

type HftResult struct {
	totalTrades int64
	totalVolume int64
	bidDepth    int64
	askDepth    int64
}

func benchHftEngine(numOrders int32) HftResult {
	var bids [maxLevels]int32
	var asks [maxLevels]int32

	var totalTrades int64 = 0
	var totalVolume int64 = 0
	var rng uint64 = 0x123456789abcdef

	for i := int32(0); i < numOrders; i++ {
		rng ^= rng << 13
		rng ^= rng >> 7
		rng ^= rng << 17

		isBuy := (rng & 1) == 1
		price := int(20 + ((rng >> 1) % 60))
		qty := int32(1 + ((rng >> 8) % 100))
		isCancel := ((rng >> 16) % 10) == 0

		if isCancel {
			if isBuy && bids[price] > 0 {
				if bids[price] > qty {
					bids[price] -= qty
				} else {
					bids[price] = 0
				}
			} else if !isBuy && asks[price] > 0 {
				if asks[price] > qty {
					asks[price] -= qty
				} else {
					asks[price] = 0
				}
			}
			continue
		}

		if isBuy {
			for p := 0; p <= price && qty > 0; p++ {
				if asks[p] > 0 {
					tradeQty := qty
					if asks[p] < tradeQty {
						tradeQty = asks[p]
					}
					asks[p] -= tradeQty
					qty -= tradeQty
					totalTrades++
					totalVolume += int64(tradeQty) * int64(p)
				}
			}
			if qty > 0 {
				bids[price] += qty
			}
		} else {
			for p := maxLevels - 1; p >= price && qty > 0; p-- {
				if bids[p] > 0 {
					tradeQty := qty
					if bids[p] < tradeQty {
						tradeQty = bids[p]
					}
					bids[p] -= tradeQty
					qty -= tradeQty
					totalTrades++
					totalVolume += int64(tradeQty) * int64(p)
				}
			}
			if qty > 0 {
				asks[price] += qty
			}
		}
	}

	var bidDepth, askDepth int64
	for i := 0; i < maxLevels; i++ {
		bidDepth += int64(bids[i])
		askDepth += int64(asks[i])
	}

	return HftResult{totalTrades, totalVolume, bidDepth, askDepth}
}

func main() {
	fmt.Println("=== BRUTAL BENCHMARK: Go (1.25.1 -ldflags=\"-s -w\") ===")

	t0 := time.Now()
	res1 := benchRaymarch(500, 500)
	t1 := time.Since(t0)
	fmt.Printf("1. Raymarcher 3D (250K rays): %.2f ms | Checksum: %d\n", float64(t1.Microseconds())/1000.0, res1)

	t0 = time.Now()
	res2 := benchBinaryTrees(16)
	t1 = time.Since(t0)
	fmt.Printf("2. Binary Trees (Depth 16):   %.2f ms | Checksum: %d\n", float64(t1.Microseconds())/1000.0, res2)

	t0 = time.Now()
	res3 := benchHftEngine(1000000)
	t1 = time.Since(t0)
	fmt.Printf("3. HFT Order Matching (1M):   %.2f ms | Trades: %d | Vol: %d\n",
		float64(t1.Microseconds())/1000.0, res3.totalTrades, res3.totalVolume)
}
