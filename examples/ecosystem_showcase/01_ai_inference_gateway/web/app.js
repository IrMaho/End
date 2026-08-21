const btnInfer = document.getElementById('btn-infer');
const btnSim = document.getElementById('btn-sim');
const aiOutput = document.getElementById('ai-output');
const inferSpeed = document.getElementById('infer-speed');
const valLatency = document.getElementById('val-latency');
const valTokens = document.getElementById('val-tokens');
const valMatMul = document.getElementById('val-matmul');

btnInfer.addEventListener('click', async () => {
    aiOutput.textContent = 'در حال ارزیابی تنسور در آرنای حافظه Zero-GC...';
    try {
        const res = await fetch('/api/ai/infer');
        const data = await res.json();
        
        aiOutput.textContent = `[End Neural Engine Response]
✅ استنتاج با موفقیت انجام شد:
مدل: ${data.model}
شتاب‌دهنده: ${data.simd_vectorization}
حافظه: ${data.memory_model}

پاسخ تولید شده:
«الگوریتم ضرب ماتریسی در زبان End با استفاده از حافظه پیوسته (Contiguous Memory) و بهره‌گیری از دستورات برداری سخت‌افزاری AVX-512 و ARM NEON بدون هیچ‌گونه سربار GC یا کپی اضافی در رم، محاسبات را با سرعت ۸۳۰ میلیون عملیات در ثانیه پردازش می‌کند.»`;

        inferSpeed.textContent = `${data.throughput_tok_per_sec.toLocaleString()} tokens/sec`;
        valLatency.textContent = `${data.total_time_us} µs`;
        valTokens.textContent = `${data.completion_tokens} tokens`;
        valMatMul.textContent = `${(data.matmul_benchmark / 1000).toFixed(1)} GFLOPS`;
    } catch (e) {
        aiOutput.textContent = 'خطا در ارتباط با گیت‌وی هوش مصنوعی End: ' + e;
    }
});

btnSim.addEventListener('click', async () => {
    aiOutput.textContent = 'در حال محاسبه شباهت کسینوسی برداری...';
    try {
        const res = await fetch('/api/ai/similarity');
        const data = await res.json();
        
        aiOutput.textContent = `[Vector Embedding Search Result]
امتیاز شباهت کسینوسی: ${data.cosine_similarity_score}
نزدیک‌ترین بردار مستند: ${data.nearest_neighbor}
ابعاد بردار: ${data.embedding_dimensions} Dimensions
زمان پاسخ محاسبه: ${data.latency_us} µs`;
        
        valLatency.textContent = `${data.latency_us} µs`;
    } catch (e) {
        aiOutput.textContent = 'خطا در ارتباط با سرور: ' + e;
    }
});

// Poll status on load
fetch('/api/ai/status').then(r => r.json()).then(data => {
    console.log('AI Gateway Live:', data);
}).catch(console.warn);
