# ⚡ End Language — M:N Fiber Concurrency & Async I/O Runtime
## Work-Stealing Green Fibers, Micro-Stacks (<4KB), and Multiplatform Non-Blocking Reactor

---

## 🌟 1. M:N Green Fiber Scheduler

The End concurrency runtime (`endc/src/runtime/mod.rs`) introduces an M:N work-stealing fiber scheduler:

- **Micro-Stacks**: Fibers start with tiny initial stacks (<4 KB) that dynamically allocate stack pages.
- **Cooperative & Preemptive Multiplexing**: Thousands of fibers multiplexed across all CPU cores with zero OS thread overhead.
- **Lock-Free MPMC Channels**: High-throughput message passing between fibers.

---

## 🌐 2. Multiplatform Async I/O Reactor

The native async reactor provides non-blocking socket handling across operating systems:
- **Windows**: IOCP (I/O Completion Ports)
- **Linux**: epoll (Edge-Triggered)
- **macOS**: kqueue

---

## 💡 Example: 100,000 Concurrent Fibers

```end
pub fn main() void {
    val chan = Channel<i64>.create(1000)
    
    for i in 100000 {
        spawn {
            chan.send(i * 2)
        }
    }
}
```
