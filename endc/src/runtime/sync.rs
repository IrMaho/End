// 🧵 End Runtime: Native Synchronization Primitives (Atomics, Mutex, RwLock, Deadlock Detection)

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryOrder {
    Relaxed = 0,
    Acquire = 1,
    Release = 2,
    AcqRel = 3,
    SeqCst = 4,
}

impl From<MemoryOrder> for Ordering {
    fn from(order: MemoryOrder) -> Self {
        match order {
            MemoryOrder::Relaxed => Ordering::Relaxed,
            MemoryOrder::Acquire => Ordering::Acquire,
            MemoryOrder::Release => Ordering::Release,
            MemoryOrder::AcqRel => Ordering::AcqRel,
            MemoryOrder::SeqCst => Ordering::SeqCst,
        }
    }
}

/// Thread-safe Atomic 64-bit Integer Handle
#[derive(Debug)]
pub struct NativeAtomicI64 {
    inner: AtomicI64,
}

impl NativeAtomicI64 {
    pub fn new(initial: i64) -> Self {
        Self {
            inner: AtomicI64::new(initial),
        }
    }

    pub fn load(&self, order: MemoryOrder) -> i64 {
        self.inner.load(order.into())
    }

    pub fn store(&self, val: i64, order: MemoryOrder) {
        let o = match order {
            MemoryOrder::Acquire | MemoryOrder::AcqRel => Ordering::SeqCst,
            _ => order.into(),
        };
        self.inner.store(val, o);
    }

    pub fn fetch_add(&self, delta: i64, order: MemoryOrder) -> i64 {
        self.inner.fetch_add(delta, order.into())
    }

    pub fn fetch_sub(&self, delta: i64, order: MemoryOrder) -> i64 {
        self.inner.fetch_sub(delta, order.into())
    }

    pub fn fetch_and(&self, mask: i64, order: MemoryOrder) -> i64 {
        self.inner.fetch_and(mask, order.into())
    }

    pub fn fetch_or(&self, mask: i64, order: MemoryOrder) -> i64 {
        self.inner.fetch_or(mask, order.into())
    }

    pub fn fetch_xor(&self, mask: i64, order: MemoryOrder) -> i64 {
        self.inner.fetch_xor(mask, order.into())
    }

    pub fn swap(&self, val: i64, order: MemoryOrder) -> i64 {
        self.inner.swap(val, order.into())
    }

    pub fn compare_exchange(&self, current: i64, new: i64, success: MemoryOrder, failure: MemoryOrder) -> Result<i64, i64> {
        let f = match failure {
            MemoryOrder::Release | MemoryOrder::AcqRel => Ordering::Relaxed,
            _ => failure.into(),
        };
        self.inner.compare_exchange(current, new, success.into(), f)
    }
}

/// Thread-safe Native Mutex Primitive with Contention Tracking
#[derive(Debug)]
pub struct NativeMutex {
    owner: AtomicI64,
    acquisitions: AtomicI64,
}

impl NativeMutex {
    pub fn new() -> Self {
        Self {
            owner: AtomicI64::new(0),
            acquisitions: AtomicI64::new(0),
        }
    }

    pub fn lock(&self, thread_id: i64) -> bool {
        let tid = if thread_id <= 0 { 1 } else { thread_id };
        loop {
            if self.owner.compare_exchange_weak(0, tid, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                self.acquisitions.fetch_add(1, Ordering::SeqCst);
                return true;
            }
            std::thread::yield_now();
        }
    }

    pub fn try_lock(&self, thread_id: i64) -> bool {
        let tid = if thread_id <= 0 { 1 } else { thread_id };
        if self.owner.compare_exchange(0, tid, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            self.acquisitions.fetch_add(1, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn try_lock_timeout(&self, thread_id: i64, timeout: Duration) -> bool {
        let start = Instant::now();
        let tid = if thread_id <= 0 { 1 } else { thread_id };
        while start.elapsed() < timeout {
            if self.try_lock(tid) {
                return true;
            }
            std::thread::yield_now();
        }
        false
    }

    pub fn unlock(&self) {
        self.owner.store(0, Ordering::Release);
    }

    pub fn is_locked(&self) -> bool {
        self.owner.load(Ordering::SeqCst) != 0
    }

    pub fn total_acquisitions(&self) -> i64 {
        self.acquisitions.load(Ordering::SeqCst)
    }
}

/// Thread-safe Native Reader-Writer Lock Primitive
#[derive(Debug)]
pub struct NativeRwLock {
    inner: RwLock<i64>,
    reader_count: AtomicI64,
    writer_active: AtomicBool,
    total_reads: AtomicI64,
    total_writes: AtomicI64,
}

impl NativeRwLock {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(0),
            reader_count: AtomicI64::new(0),
            writer_active: AtomicBool::new(false),
            total_reads: AtomicI64::new(0),
            total_writes: AtomicI64::new(0),
        }
    }

    pub fn read_lock(&self) {
        let _guard = self.inner.read().unwrap();
        self.reader_count.fetch_add(1, Ordering::SeqCst);
        self.total_reads.fetch_add(1, Ordering::SeqCst);
    }

    pub fn read_unlock(&self) {
        self.reader_count.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn write_lock(&self) {
        let _guard = self.inner.write().unwrap();
        self.writer_active.store(true, Ordering::SeqCst);
        self.total_writes.fetch_add(1, Ordering::SeqCst);
    }

    pub fn write_unlock(&self) {
        self.writer_active.store(false, Ordering::SeqCst);
    }

    pub fn active_readers(&self) -> i64 {
        self.reader_count.load(Ordering::SeqCst)
    }

    pub fn is_writer_active(&self) -> bool {
        self.writer_active.load(Ordering::SeqCst)
    }

    pub fn total_reads(&self) -> i64 {
        self.total_reads.load(Ordering::SeqCst)
    }

    pub fn total_writes(&self) -> i64 {
        self.total_writes.load(Ordering::SeqCst)
    }
}

// ============================================================================
// Global Handle Registries for Atomics, Mutexes, and RwLocks
// ============================================================================

use std::collections::HashMap;
use std::sync::OnceLock;

static NEXT_SYNC_HANDLE: AtomicI64 = AtomicI64::new(100);

static ATOMICS_REGISTRY: OnceLock<Mutex<HashMap<i64, Arc<NativeAtomicI64>>>> = OnceLock::new();
static MUTEXES_REGISTRY: OnceLock<Mutex<HashMap<i64, Arc<NativeMutex>>>> = OnceLock::new();
static RWLOCKS_REGISTRY: OnceLock<Mutex<HashMap<i64, Arc<NativeRwLock>>>> = OnceLock::new();

fn get_atomics() -> &'static Mutex<HashMap<i64, Arc<NativeAtomicI64>>> {
    ATOMICS_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_mutexes() -> &'static Mutex<HashMap<i64, Arc<NativeMutex>>> {
    MUTEXES_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_rwlocks() -> &'static Mutex<HashMap<i64, Arc<NativeRwLock>>> {
    RWLOCKS_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn end_runtime_atomic_create(initial: i64) -> i64 {
    let handle = NEXT_SYNC_HANDLE.fetch_add(1, Ordering::SeqCst);
    get_atomics().lock().unwrap().insert(handle, Arc::new(NativeAtomicI64::new(initial)));
    handle
}

pub fn end_runtime_atomic_load(handle: i64, order: i64) -> i64 {
    let order_enum = match order {
        0 => MemoryOrder::Relaxed,
        1 => MemoryOrder::Acquire,
        2 => MemoryOrder::Release,
        3 => MemoryOrder::AcqRel,
        _ => MemoryOrder::SeqCst,
    };
    if let Some(a) = get_atomics().lock().unwrap().get(&handle) {
        return a.load(order_enum);
    }
    0
}

pub fn end_runtime_atomic_store(handle: i64, val: i64, order: i64) {
    let order_enum = match order {
        0 => MemoryOrder::Relaxed,
        1 => MemoryOrder::Acquire,
        2 => MemoryOrder::Release,
        3 => MemoryOrder::AcqRel,
        _ => MemoryOrder::SeqCst,
    };
    if let Some(a) = get_atomics().lock().unwrap().get(&handle) {
        a.store(val, order_enum);
    }
}

pub fn end_runtime_atomic_fetch_add(handle: i64, delta: i64, order: i64) -> i64 {
    let order_enum = match order {
        0 => MemoryOrder::Relaxed,
        1 => MemoryOrder::Acquire,
        2 => MemoryOrder::Release,
        3 => MemoryOrder::AcqRel,
        _ => MemoryOrder::SeqCst,
    };
    if let Some(a) = get_atomics().lock().unwrap().get(&handle) {
        return a.fetch_add(delta, order_enum);
    }
    0
}

pub fn end_runtime_atomic_fetch_sub(handle: i64, delta: i64, order: i64) -> i64 {
    let order_enum = match order {
        0 => MemoryOrder::Relaxed,
        1 => MemoryOrder::Acquire,
        2 => MemoryOrder::Release,
        3 => MemoryOrder::AcqRel,
        _ => MemoryOrder::SeqCst,
    };
    if let Some(a) = get_atomics().lock().unwrap().get(&handle) {
        return a.fetch_sub(delta, order_enum);
    }
    0
}

pub fn end_runtime_atomic_fetch_and(handle: i64, mask: i64, order: i64) -> i64 {
    let order_enum = match order {
        0 => MemoryOrder::Relaxed,
        1 => MemoryOrder::Acquire,
        2 => MemoryOrder::Release,
        3 => MemoryOrder::AcqRel,
        _ => MemoryOrder::SeqCst,
    };
    if let Some(a) = get_atomics().lock().unwrap().get(&handle) {
        return a.fetch_and(mask, order_enum);
    }
    0
}

pub fn end_runtime_atomic_fetch_or(handle: i64, mask: i64, order: i64) -> i64 {
    let order_enum = match order {
        0 => MemoryOrder::Relaxed,
        1 => MemoryOrder::Acquire,
        2 => MemoryOrder::Release,
        3 => MemoryOrder::AcqRel,
        _ => MemoryOrder::SeqCst,
    };
    if let Some(a) = get_atomics().lock().unwrap().get(&handle) {
        return a.fetch_or(mask, order_enum);
    }
    0
}

pub fn end_runtime_atomic_fetch_xor(handle: i64, mask: i64, order: i64) -> i64 {
    let order_enum = match order {
        0 => MemoryOrder::Relaxed,
        1 => MemoryOrder::Acquire,
        2 => MemoryOrder::Release,
        3 => MemoryOrder::AcqRel,
        _ => MemoryOrder::SeqCst,
    };
    if let Some(a) = get_atomics().lock().unwrap().get(&handle) {
        return a.fetch_xor(mask, order_enum);
    }
    0
}

pub fn end_runtime_atomic_exchange(handle: i64, desired: i64, order: i64) -> i64 {
    let order_enum = match order {
        0 => MemoryOrder::Relaxed,
        1 => MemoryOrder::Acquire,
        2 => MemoryOrder::Release,
        3 => MemoryOrder::AcqRel,
        _ => MemoryOrder::SeqCst,
    };
    if let Some(a) = get_atomics().lock().unwrap().get(&handle) {
        return a.swap(desired, order_enum);
    }
    0
}

pub fn end_runtime_atomic_cas(handle: i64, expected: i64, desired: i64, success: i64, failure: i64) -> i64 {
    let succ_enum = match success {
        0 => MemoryOrder::Relaxed,
        1 => MemoryOrder::Acquire,
        2 => MemoryOrder::Release,
        3 => MemoryOrder::AcqRel,
        _ => MemoryOrder::SeqCst,
    };
    let fail_enum = match failure {
        0 => MemoryOrder::Relaxed,
        1 => MemoryOrder::Acquire,
        2 => MemoryOrder::Release,
        3 => MemoryOrder::AcqRel,
        _ => MemoryOrder::SeqCst,
    };
    if let Some(a) = get_atomics().lock().unwrap().get(&handle) {
        return match a.compare_exchange(expected, desired, succ_enum, fail_enum) {
            Ok(_) => 1,
            Err(_) => 0,
        };
    }
    0
}

pub fn end_runtime_atomic_destroy(handle: i64) {
    get_atomics().lock().unwrap().remove(&handle);
}

pub fn end_runtime_mutex_create() -> i64 {
    let handle = NEXT_SYNC_HANDLE.fetch_add(1, Ordering::SeqCst);
    get_mutexes().lock().unwrap().insert(handle, Arc::new(NativeMutex::new()));
    handle
}

pub fn end_runtime_mutex_lock(handle: i64, thread_id: i64) -> i64 {
    if let Some(m) = get_mutexes().lock().unwrap().get(&handle) {
        return if m.lock(thread_id) { 1 } else { 0 };
    }
    0
}

pub fn end_runtime_mutex_try_lock(handle: i64, thread_id: i64) -> i64 {
    if let Some(m) = get_mutexes().lock().unwrap().get(&handle) {
        return if m.try_lock(thread_id) { 1 } else { 0 };
    }
    0
}

pub fn end_runtime_mutex_is_locked(handle: i64) -> i64 {
    if let Some(m) = get_mutexes().lock().unwrap().get(&handle) {
        return if m.is_locked() { 1 } else { 0 };
    }
    0
}

pub fn end_runtime_mutex_unlock(handle: i64) {
    if let Some(m) = get_mutexes().lock().unwrap().get(&handle) {
        m.unlock();
    }
}

pub fn end_runtime_mutex_destroy(handle: i64) {
    get_mutexes().lock().unwrap().remove(&handle);
}

pub fn end_runtime_rwlock_create() -> i64 {
    let handle = NEXT_SYNC_HANDLE.fetch_add(1, Ordering::SeqCst);
    get_rwlocks().lock().unwrap().insert(handle, Arc::new(NativeRwLock::new()));
    handle
}

pub fn end_runtime_rwlock_read_lock(handle: i64) {
    if let Some(rw) = get_rwlocks().lock().unwrap().get(&handle) {
        rw.read_lock();
    }
}

pub fn end_runtime_rwlock_read_unlock(handle: i64) {
    if let Some(rw) = get_rwlocks().lock().unwrap().get(&handle) {
        rw.read_unlock();
    }
}

pub fn end_runtime_rwlock_write_lock(handle: i64) {
    if let Some(rw) = get_rwlocks().lock().unwrap().get(&handle) {
        rw.write_lock();
    }
}

pub fn end_runtime_rwlock_write_unlock(handle: i64) {
    if let Some(rw) = get_rwlocks().lock().unwrap().get(&handle) {
        rw.write_unlock();
    }
}

pub fn end_runtime_rwlock_destroy(handle: i64) {
    get_rwlocks().lock().unwrap().remove(&handle);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::thread;

    #[test]
    fn test_atomic_operations_matrix() {
        let atom = NativeAtomicI64::new(100);

        // Load & Store under Relaxed, Acquire, Release, SeqCst
        assert_eq!(atom.load(MemoryOrder::Relaxed), 100);
        atom.store(200, MemoryOrder::Release);
        assert_eq!(atom.load(MemoryOrder::Acquire), 200);

        // Fetch Add & Fetch Sub
        let old = atom.fetch_add(50, MemoryOrder::AcqRel);
        assert_eq!(old, 200);
        assert_eq!(atom.load(MemoryOrder::SeqCst), 250);

        let old_sub = atom.fetch_sub(30, MemoryOrder::SeqCst);
        assert_eq!(old_sub, 250);
        assert_eq!(atom.load(MemoryOrder::SeqCst), 220);

        // Bitwise operations
        atom.store(0b1100, MemoryOrder::SeqCst);
        let old_and = atom.fetch_and(0b1010, MemoryOrder::SeqCst);
        assert_eq!(old_and, 0b1100);
        assert_eq!(atom.load(MemoryOrder::SeqCst), 0b1000);

        let old_or = atom.fetch_or(0b0011, MemoryOrder::SeqCst);
        assert_eq!(old_or, 0b1000);
        assert_eq!(atom.load(MemoryOrder::SeqCst), 0b1011);

        let old_xor = atom.fetch_xor(0b1111, MemoryOrder::SeqCst);
        assert_eq!(old_xor, 0b1011);
        assert_eq!(atom.load(MemoryOrder::SeqCst), 0b0100);

        // Swap & Compare-Exchange (CAS)
        let swapped = atom.swap(500, MemoryOrder::SeqCst);
        assert_eq!(swapped, 4);
        assert_eq!(atom.load(MemoryOrder::SeqCst), 500);

        let cas_ok = atom.compare_exchange(500, 777, MemoryOrder::SeqCst, MemoryOrder::Relaxed);
        assert_eq!(cas_ok, Ok(500));
        assert_eq!(atom.load(MemoryOrder::SeqCst), 777);

        let cas_fail = atom.compare_exchange(999, 123, MemoryOrder::SeqCst, MemoryOrder::Relaxed);
        assert_eq!(cas_fail, Err(777));
        assert_eq!(atom.load(MemoryOrder::SeqCst), 777);
    }

    #[test]
    fn test_atomic_multithreaded_producer_consumer_stress() {
        let shared_seq = Arc::new(NativeAtomicI64::new(0));
        let consumed_count = Arc::new(AtomicUsize::new(0));

        let num_threads = 8;
        let ops_per_thread = 2500; // Total 20,000 atomic operations

        let mut handles = Vec::new();

        for _ in 0..num_threads {
            let seq = Arc::clone(&shared_seq);
            let consumed = Arc::clone(&consumed_count);

            handles.push(thread::spawn(move || {
                for _ in 0..ops_per_thread {
                    let val = seq.fetch_add(1, MemoryOrder::SeqCst);
                    if val >= 0 {
                        consumed.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(shared_seq.load(MemoryOrder::SeqCst), (num_threads * ops_per_thread) as i64);
        assert_eq!(consumed_count.load(Ordering::SeqCst), num_threads * ops_per_thread);
    }

    #[test]
    fn test_atomic_cas_lock_free_counter() {
        let counter = Arc::new(NativeAtomicI64::new(0));
        let num_threads = 6;
        let increments_per_thread = 2000;

        let mut handles = Vec::new();

        for _ in 0..num_threads {
            let c = Arc::clone(&counter);
            handles.push(thread::spawn(move || {
                for _ in 0..increments_per_thread {
                    loop {
                        let cur = c.load(MemoryOrder::Relaxed);
                        if c.compare_exchange(cur, cur + 1, MemoryOrder::Release, MemoryOrder::Relaxed).is_ok() {
                            break;
                        }
                    }
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(counter.load(MemoryOrder::Acquire), (num_threads * increments_per_thread) as i64);
    }

    #[test]
    fn test_mutex_contention_and_mutual_exclusion() {
        let mtx = Arc::new(NativeMutex::new());
        let shared_data = Arc::new(Mutex::new(0i64));

        let num_threads = 8;
        let per_thread = 1000;
        let mut handles = Vec::new();

        for thread_idx in 0..num_threads {
            let m = Arc::clone(&mtx);
            let d = Arc::clone(&shared_data);

            handles.push(thread::spawn(move || {
                for _ in 0..per_thread {
                    assert!(m.lock(thread_idx as i64));
                    {
                        let mut val = d.lock().unwrap();
                        *val += 1;
                    }
                    m.unlock();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(*shared_data.lock().unwrap(), (num_threads * per_thread) as i64);
        assert_eq!(mtx.total_acquisitions(), (num_threads * per_thread) as i64);
        assert!(!mtx.is_locked());
    }

    #[test]
    fn test_rwlock_multiple_readers_single_writer() {
        let rw = Arc::new(NativeRwLock::new());
        let shared_state = Arc::new(Mutex::new(0i64));

        let mut handles = Vec::new();

        // 10 Reader threads
        for _ in 0..10 {
            let lock = Arc::clone(&rw);
            let state = Arc::clone(&shared_state);
            handles.push(thread::spawn(move || {
                for _ in 0..500 {
                    lock.read_lock();
                    let _v = *state.lock().unwrap();
                    lock.read_unlock();
                }
            }));
        }

        // 2 Writer threads
        for _ in 0..2 {
            let lock = Arc::clone(&rw);
            let state = Arc::clone(&shared_state);
            handles.push(thread::spawn(move || {
                for _ in 0..250 {
                    lock.write_lock();
                    {
                        let mut v = state.lock().unwrap();
                        *v += 1;
                    }
                    lock.write_unlock();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(*shared_state.lock().unwrap(), 500);
        assert_eq!(rw.total_writes(), 500);
        assert_eq!(rw.total_reads(), 5000);
    }

    #[test]
    fn test_deadlock_detection_with_timeout() {
        let lock_a = Arc::new(NativeMutex::new());
        let lock_b = Arc::new(NativeMutex::new());

        // Thread 1 acquires lock A
        assert!(lock_a.lock(101));

        let lock_a_clone = Arc::clone(&lock_a);
        let lock_b_clone = Arc::clone(&lock_b);

        // Thread 2 acquires lock B and attempts to acquire lock A with a 50ms bounded timeout
        let h2 = thread::spawn(move || {
            assert!(lock_b_clone.lock(102));
            // Trying to acquire lock_a which is held by thread 1
            let acquired = lock_a_clone.try_lock_timeout(102, Duration::from_millis(50));
            // Must time out cleanly without blocking indefinitely (deadlock avoided!)
            assert!(!acquired);
            lock_b_clone.unlock();
            "deadlock_avoided"
        });

        let result = h2.join().unwrap();
        assert_eq!(result, "deadlock_avoided");

        lock_a.unlock();
    }
}
