const btnSweep = document.getElementById('btn-sweep');
const btnRing = document.getElementById('btn-ring');
const patrolOutput = document.getElementById('patrol-output');
const valP99 = document.getElementById('val-p99');
const valRing = document.getElementById('val-ring');

btnSweep.addEventListener('click', async () => {
    patrolOutput.textContent = 'در حال ارسال پروب‌های پایش نانوثانیه‌ای و اسکن نشت حافظه نودها...';
    try {
        const res = await fetch('/api/mesh/patrol');
        const data = await res.json();

        patrolOutput.textContent = `[AUTONOMOUS PATROL SWEEP #${data.sweep_id}]
✅ سلامت کلاستر با موفقیت ممیزی شد:
نودهای بررسی‌شده: ${data.inspected_nodes} سرویس فعال
نشت حافظه: ${data.memory_leaks} Bytes (Zero-Leak Strict Mode)
خطاهای کشف‌نشده: ${data.unhandled_panics} Panics
وضعیت حرارتی پردازنده: ${data.cpu_thermal_status}
تاخیر پایش (Duration): ${data.patrol_duration_us} µs
شاخص P99 Latency: ${data.p99_latency_us} µs`;

        valP99.textContent = `${data.p99_latency_us} µs`;
    } catch (e) {
        patrolOutput.textContent = 'خطا در اجرای Patrol Sweep: ' + e;
    }
});

btnRing.addEventListener('click', async () => {
    try {
        const res = await fetch('/api/mesh/ring');
        const data = await res.json();

        patrolOutput.textContent = `[ZERO-ALLOC RING BUFFER EVENT]
📥 رویداد جدید به بافر حلقوی در حافظه رم تزریق شد:
ظرفیت کل بافر: ${data.capacity.toLocaleString()} آیتم
تعداد رویدادهای در صف: ${data.buffered_events.toLocaleString()}
تخصیص حافظه اضافه: ${data.zero_alloc_bytes} Bytes (Zero-Alloc Overhead)
سیاست سرریز: ${data.eviction_policy}`;

        valRing.textContent = `${data.buffered_events} / ${data.capacity}`;
    } catch (e) {
        console.warn('Ring Error:', e);
    }
});

// Initial sweep
btnSweep.click();
