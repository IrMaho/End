package main

import (
	"fmt"
	"math"
	"math/bits"
	"os"
	"strconv"
	"time"
)

func splitmix64(state *uint64) uint64 {
	*state += 0x9E3779B97F4A7C15
	z := *state
	z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9
	z = (z ^ (z >> 27)) * 0x94D049BB133111EB
	return z ^ (z >> 31)
}

// 1. Raymarcher
type Vec3 struct{ x, y, z float32 }

func vAdd(a, b Vec3) Vec3    { return Vec3{a.x + b.x, a.y + b.y, a.z + b.z} }
func vSub(a, b Vec3) Vec3    { return Vec3{a.x - b.x, a.y - b.y, a.z - b.z} }
func vScale(a Vec3, s float32) Vec3 { return Vec3{a.x * s, a.y * s, a.z * s} }
func vDot(a, b Vec3) float32 { return a.x*b.x + a.y*b.y + a.z*b.z }
func vLength(a Vec3) float32 { return float32(math.Sqrt(float64(vDot(a, a)))) }
func vNorm(a Vec3) Vec3 {
	l := vLength(a)
	if l > 0.00001 {
		return vScale(a, 1.0/l)
	}
	return a
}

func sdfSphere(p Vec3, r float32) float32 { return vLength(p) - r }
func sdfTorus(p Vec3, r1, r2 float32) float32 {
	qx := float32(math.Sqrt(float64(p.x*p.x+p.z*p.z))) - r1
	return float32(math.Sqrt(float64(qx*qx+p.y*p.y))) - r2
}
func sdfScene(p Vec3) float32 {
	dSphere := sdfSphere(vSub(p, Vec3{0, 1, 0}), 1.0)
	dTorus := sdfTorus(vSub(p, Vec3{0, 0.5, 0}), 1.2, 0.3)
	dFloor := p.y
	d := dSphere
	if dTorus < d {
		d = dTorus
	}
	if dFloor < d {
		d = dFloor
	}
	return d
}
func calcNormal(p Vec3) Vec3 {
	eps := float32(0.001)
	return vNorm(Vec3{
		sdfScene(Vec3{p.x + eps, p.y, p.z}) - sdfScene(Vec3{p.x - eps, p.y, p.z}),
		sdfScene(Vec3{p.x, p.y + eps, p.z}) - sdfScene(Vec3{p.x, p.y - eps, p.z}),
		sdfScene(Vec3{p.x, p.y, p.z + eps}) - sdfScene(Vec3{p.x, p.y, p.z - eps}),
	})
}

func bench1Raymarcher() int64 {
	w, h := 500, 500
	var checksum int64 = 0
	ro := Vec3{0, 1.5, -3.5}
	lightPos := Vec3{2, 4, -2}

	for y := 0; y < h; y++ {
		for x := 0; x < w; x++ {
			u := (2.0*float32(x) - float32(w)) / float32(h)
			v := -(2.0*float32(y) - float32(h)) / float32(h)
			rd := vNorm(Vec3{u, v, 1.5})
			var t float32 = 0.0
			hit := false
			for step := 0; step < 64; step++ {
				p := vAdd(ro, vScale(rd, t))
				d := sdfScene(p)
				if d < 0.001 {
					n := calcNormal(p)
					l := vNorm(vSub(lightPos, p))
					diff := vDot(n, l)
					if diff < 0.0 {
						diff = 0.0
					}
					color := int64(diff * 255.0)
					checksum += color
					hit = true
					break
				}
				t += d
				if t > 20.0 {
					break
				}
			}
			if !hit {
				checksum += 10
			}
		}
	}
	return checksum
}

// 2. Binary Trees
type TreeNode struct {
	left, right *TreeNode
	val         int32
}

func createTree(depth int32) *TreeNode {
	node := &TreeNode{val: depth}
	if depth > 0 {
		node.left = createTree(depth - 1)
		node.right = createTree(depth - 1)
	}
	return node
}
func sumTree(node *TreeNode) int64 {
	if node == nil {
		return 0
	}
	return int64(node.val) + sumTree(node.left) - sumTree(node.right)
}

func bench2BinaryTrees() int64 {
	var maxDepth int32 = 16
	stretch := createTree(maxDepth + 1)
	check := sumTree(stretch)

	longLived := createTree(maxDepth)
	for depth := int32(4); depth <= maxDepth; depth += 2 {
		iterations := 1 << (maxDepth - depth + 4)
		for i := 1; i <= iterations; i++ {
			t1 := createTree(depth)
			check += sumTree(t1)
		}
	}
	check += sumTree(longLived)
	return check
}

// 3. HFT Engine
func bench3HftEngine() int64 {
	var rng uint64 = 0x123456789ABCDEF0
	var totalVolume int64 = 0
	var buyDepth [100]int32
	var sellDepth [100]int32

	for i := 0; i < 1000000; i++ {
		r := splitmix64(&rng)
		side := (r >> 63) & 1
		price := int(r % 100)
		qty := int32(((r >> 8) % 50) + 1)
		op := (r >> 16) % 10

		if op == 0 {
			if side == 0 {
				buyDepth[price] = 0
			} else {
				sellDepth[price] = 0
			}
		} else if side == 0 {
			for p := price; p >= 0 && qty > 0; p-- {
				if sellDepth[p] > 0 {
					fill := qty
					if sellDepth[p] < fill {
						fill = sellDepth[p]
					}
					sellDepth[p] -= fill
					qty -= fill
					totalVolume += int64(fill) * int64(p+1)
				}
			}
			if qty > 0 {
				buyDepth[price] += qty
			}
		} else {
			for p := price; p < 100 && qty > 0; p++ {
				if buyDepth[p] > 0 {
					fill := qty
					if buyDepth[p] < fill {
						fill = buyDepth[p]
					}
					buyDepth[p] -= fill
					qty -= fill
					totalVolume += int64(fill) * int64(p+1)
				}
			}
			if qty > 0 {
				sellDepth[price] += qty
			}
		}
	}
	return totalVolume
}

// 4. SHA-256
var K256 = [64]uint32{
	0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
	0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
	0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
	0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
	0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
	0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
	0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
	0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0x0bef9a3f, 0xc67178f2,
}

func bench4Sha256() int64 {
	state := [8]uint32{
		0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
		0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
	}
	var w [64]uint32
	var prng uint64 = 0xCAFEBABE12345678

	for block := 0; block < 500000; block++ {
		for i := 0; i < 16; i++ {
			w[i] = uint32(splitmix64(&prng))
		}
		for i := 16; i < 64; i++ {
			s0 := bits.RotateLeft32(w[i-15], -7) ^ bits.RotateLeft32(w[i-15], -18) ^ (w[i-15] >> 3)
			s1 := bits.RotateLeft32(w[i-2], -17) ^ bits.RotateLeft32(w[i-2], -19) ^ (w[i-2] >> 10)
			w[i] = w[i-16] + s0 + w[i-7] + s1
		}
		a, b, c, d := state[0], state[1], state[2], state[3]
		e, f, g, h := state[4], state[5], state[6], state[7]

		for i := 0; i < 64; i++ {
			s1 := bits.RotateLeft32(e, -6) ^ bits.RotateLeft32(e, -11) ^ bits.RotateLeft32(e, -25)
			ch := (e & f) ^ ((^e) & g)
			temp1 := h + s1 + ch + K256[i] + w[i]
			s0 := bits.RotateLeft32(a, -2) ^ bits.RotateLeft32(a, -13) ^ bits.RotateLeft32(a, -22)
			maj := (a & b) ^ (a & c) ^ (b & c)
			temp2 := s0 + maj

			h, g, f, e = g, f, e, d+temp1
			d, c, b, a = c, b, a, temp1+temp2
		}
		state[0] += a
		state[1] += b
		state[2] += c
		state[3] += d
		state[4] += e
		state[5] += f
		state[6] += g
		state[7] += h
	}
	res := (uint64(state[0]) << 32) | uint64(state[7])
	return int64(res)
}

// 5. N-Body
func bench5Nbody() int64 {
	n, steps := 1000, 1000
	posX := make([]float32, 1000)
	posY := make([]float32, 1000)
	posZ := make([]float32, 1000)
	velX := make([]float32, 1000)
	velY := make([]float32, 1000)
	velZ := make([]float32, 1000)
	mass := make([]float32, 1000)
	var prng uint64 = 0x5555AAAA5555AAAA

	for i := 0; i < n; i++ {
		posX[i] = (float32(splitmix64(&prng)%1000) / 100.0) - 5.0
		posY[i] = (float32(splitmix64(&prng)%1000) / 100.0) - 5.0
		posZ[i] = (float32(splitmix64(&prng)%1000) / 100.0) - 5.0
		mass[i] = 1.0 + (float32(splitmix64(&prng)%100) / 10.0)
	}

	dt := float32(0.01)
	eps2 := float32(0.001)

	for step := 0; step < steps; step++ {
		for i := 0; i < n; i++ {
			var fx, fy, fz float32
			for j := 0; j < n; j++ {
				if i == j {
					continue
				}
				dx := posX[j] - posX[i]
				dy := posY[j] - posY[i]
				dz := posZ[j] - posZ[i]
				distSq := dx*dx + dy*dy + dz*dz + eps2
				distInv := float32(1.0 / math.Sqrt(float64(distSq)))
				f := mass[j] * (distInv * distInv * distInv)
				fx += dx * f
				fy += dy * f
				fz += dz * f
			}
			velX[i] += fx * dt
			velY[i] += fy * dt
			velZ[i] += fz * dt
		}
		for i := 0; i < n; i++ {
			posX[i] += velX[i] * dt
			posY[i] += velY[i] * dt
			posZ[i] += velZ[i] * dt
		}
	}

	totalKe := 0.0
	for i := 0; i < n; i++ {
		totalKe += 0.5 * float64(mass[i]) * float64(velX[i]*velX[i]+velY[i]*velY[i]+velZ[i]*velZ[i])
	}
	return int64(totalKe * 1000.0)
}

// 6. Ring Buffer
func bench6RingBuffer() int64 {
	const Capacity = 65536
	const Mask = Capacity - 1
	const TotalMsgs = 10000000
	ring := make([]int64, Capacity)
	var totalSum int64 = 0

	head, tail := 0, 0
	for chunk := 0; chunk < TotalMsgs; chunk += 64 {
		for k := 0; k < 64; k++ {
			ring[(tail+k)&Mask] = int64(chunk+k)*31 + 17
		}
		tail += 64
		for k := 0; k < 64; k++ {
			totalSum += ring[(head+k)&Mask]
		}
		head += 64
	}
	return totalSum
}

// 7. DNA Levenshtein
func bench7DnaAlignment() int64 {
	const N = 1000
	dp := make([]int32, 1001)
	var prng uint64 = 0x9999888877776666
	s1 := make([]byte, 1000)
	s2 := make([]byte, 1000)
	bases := "ACGT"

	var totalDistance int64 = 0
	for pair := 0; pair < 1000; pair++ {
		for i := 0; i < N; i++ {
			s1[i] = bases[splitmix64(&prng)%4]
			s2[i] = bases[splitmix64(&prng)%4]
		}
		for j := 0; j <= N; j++ {
			dp[j] = int32(j)
		}

		for i := 1; i <= N; i++ {
			prev := dp[0]
			dp[0] = int32(i)
			for j := 1; j <= N; j++ {
				temp := dp[j]
				cost := int32(1)
				if s1[i-1] == s2[j-1] {
					cost = 0
				}
				d1 := dp[j-1] + 1
				d2 := dp[j] + 1
				d3 := prev + cost
				minD := d1
				if d2 < minD {
					minD = d2
				}
				if d3 < minD {
					minD = d3
				}
				dp[j] = minD
				prev = temp
			}
		}
		totalDistance += int64(dp[N])
	}
	return totalDistance
}

// 8. JSON Microservice
func bench8JsonSerializer() int64 {
	var hash int64 = 0
	for i := 0; i < 100000; i++ {
		s := fmt.Sprintf("{\"id\":%d,\"status\":\"active\",\"latency_us\":%d,\"tags\":[\"prod\",\"edge\",\"v2\"],\"metrics\":{\"cpu\":%.1f,\"mem\":%.1f}}",
			i, (i*37)%500, 42.5+float32(i%10), 128.4+float32(i%50))
		b := []byte(s)
		l := len(b)
		hash = hash*31 + int64(l) + int64(b[l/2])
	}
	return hash
}

// 9. FSM Lexer
func bench9FsmLexer() int64 {
	sample := "pub fn calculate_metrics(id: u64, active: bool) -> i64 { val base = id * 31; ret base + 10; } "
	sampleLen := len(sample)
	var tokenCount int64 = 0
	var tokenHash int64 = 0

	const (
		StateStart = iota
		StateIdent
		StateNumber
		StateOp
	)
	st := StateStart

	for i := 0; i < 10000000; i++ {
		c := sample[i%sampleLen]
		switch st {
		case StateStart:
			if (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_' {
				st = StateIdent
			} else if c >= '0' && c <= '9' {
				st = StateNumber
			} else if c != ' ' && c != '\n' && c != '\t' {
				st = StateOp
			}
		case StateIdent:
			if !((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_') {
				tokenCount++
				tokenHash = tokenHash*33 + 1
				st = StateStart
			}
		case StateNumber:
			if !(c >= '0' && c <= '9') {
				tokenCount++
				tokenHash = tokenHash*33 + 2
				st = StateStart
			}
		case StateOp:
			tokenCount++
			tokenHash = tokenHash*33 + 3
			st = StateStart
		}
	}
	return tokenHash + tokenCount
}

// 10. GEMM Matrix
func bench10GemmMatrix() int64 {
	const N = 512
	A := make([]float64, N*N)
	B := make([]float64, N*N)
	C := make([]float64, N*N)

	for idx := 0; idx < N*N; idx++ {
		A[idx] = float64(idx%100) * 0.01
		B[idx] = float64((idx*3)%100) * 0.01
	}

	const Block = 32
	for sj := 0; sj < N; sj += Block {
		for si := 0; si < N; si += Block {
			for sk := 0; sk < N; sk += Block {
				for i := si; i < si+Block; i++ {
					for k := sk; k < sk+Block; k++ {
						aIk := A[i*N+k]
						for j := sj; j < sj+Block; j++ {
							C[i*N+j] += aIk * B[k*N+j]
						}
					}
				}
			}
		}
	}

	trace := 0.0
	for i := 0; i < N; i++ {
		trace += C[i*N+i]
	}
	return int64(trace * 100.0)
}

// 11. Monte Carlo Black-Scholes
func bench11MonteCarlo() int64 {
	paths := 2000000
	S0, K, T, r, sigma := 100.0, 100.0, 1.0, 0.05, 0.20
	drift := (r - 0.5*sigma*sigma) * T
	vol := sigma * math.Sqrt(T)
	discount := math.Exp(-r * T)

	var prng uint64 = 0xFEEDFACECAFE1234
	totalPayoff := 0.0

	for i := 0; i < paths; i += 2 {
		u1 := float64((splitmix64(&prng)>>11)+1) / 9007199254740992.0
		u2 := float64((splitmix64(&prng)>>11)+1) / 9007199254740992.0
		radius := math.Sqrt(-2.0 * math.Log(u1))
		theta := 2.0 * math.Pi * u2
		z1 := radius * math.Cos(theta)
		z2 := radius * math.Sin(theta)

		sT1 := S0 * math.Exp(drift+vol*z1)
		sT2 := S0 * math.Exp(drift+vol*z2)

		payoff1 := 0.0
		if sT1 > K {
			payoff1 = sT1 - K
		}
		payoff2 := 0.0
		if sT2 > K {
			payoff2 = sT2 - K
		}

		totalPayoff += payoff1 + payoff2
	}
	optionPrice := (totalPayoff / float64(paths)) * discount
	return int64(optionPrice * 1000000.0)
}

// 12. Super-Scalar Reduction
type Req12 struct {
	id          uint64
	payloadSize int32
	checksum    int64
}

func processReq12(id uint64, size int32) Req12 {
	hash := id ^ 0x9E3779B97F4A7C15
	for j := uint64(0); j < 50; j++ {
		hash ^= (hash << 13)
		hash ^= (hash >> 7)
		hash ^= (hash << 17)
		hash += j + 0xBF58476D1CE4E5B9
	}
	return Req12{id: id, payloadSize: size, checksum: int64(hash)}
}

func bench12Reduction() int64 {
	const iterations = 10000000
	var sum0, sum1, sum2, sum3 int64

	for i := 0; i < iterations; i += 4 {
		sum0 += processReq12(uint64(i), 256).checksum
		sum1 += processReq12(uint64(i+1), 256).checksum
		sum2 += processReq12(uint64(i+2), 256).checksum
		sum3 += processReq12(uint64(i+3), 256).checksum
	}
	return sum0 + sum1 + sum2 + sum3
}

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: suite12_go.exe <id (1..12)>")
		return
	}
	id, _ := strconv.Atoi(os.Args[1])
	t0 := time.Now()
	var check int64
	switch id {
	case 1:
		check = bench1Raymarcher()
	case 2:
		check = bench2BinaryTrees()
	case 3:
		check = bench3HftEngine()
	case 4:
		check = bench4Sha256()
	case 5:
		check = bench5Nbody()
	case 6:
		check = bench6RingBuffer()
	case 7:
		check = bench7DnaAlignment()
	case 8:
		check = bench8JsonSerializer()
	case 9:
		check = bench9FsmLexer()
	case 10:
		check = bench10GemmMatrix()
	case 11:
		check = bench11MonteCarlo()
	case 12:
		check = bench12Reduction()
	default:
		return
	}
	elapsed := time.Since(t0)
	ms := float64(elapsed.Nanoseconds()) / 1000000.0
	fmt.Printf("RESULT:bench=%d,time_ms=%.3f,checksum=%d\n", id, ms, check)
}
