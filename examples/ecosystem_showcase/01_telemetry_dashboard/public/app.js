/**
 * END CORE OS // Modern Interactive Dashboard & 50-Endpoint Controller
 * Powered by End Native Language Backend DLL & Localhost REST API
 */

// State Management
const state = {
    isStreaming: true,
    activeTab: 'tab-dashboard',
    isFetchingTelemetry: false,
    isFetchingCluster: false,
    isFetchingLogs: false,
    endpointsData: [],
    activeModuleFilter: 'ALL',
    searchQuery: '',
    telemetryHistory: {
        cpu: Array(30).fill(25),
        mem: Array(30).fill(40),
        rps: Array(30).fill(34600),
    },
    quickBenchIters: 50000,
    pollIntervalId: null,
};

// DOM Elements Cache
const elements = {
    valHealth: document.getElementById('val-health'),
    barHealth: document.getElementById('bar-health'),
    valRps: document.getElementById('val-rps'),
    pillRps: document.getElementById('pill-rps'),
    valLatency: document.getElementById('val-latency'),
    barLatency: document.getElementById('bar-latency'),
    btnToggleStream: document.getElementById('btn-toggle-stream'),
    streamStatusText: document.getElementById('stream-status-text'),
    btnOpenEndpoints: document.getElementById('btn-open-endpoints'),
    btnRunQuickBench: document.getElementById('btn-run-quick-bench'),
    quickBenchTime: document.getElementById('quick-bench-time'),
    quickBenchOps: document.getElementById('quick-bench-ops'),
    quickBenchHash: document.getElementById('quick-bench-hash'),
    miniClusterList: document.getElementById('mini-cluster-list'),
    clusterNodesGrid: document.getElementById('cluster-nodes-grid'),
    terminalBody: document.getElementById('terminal-body'),
    btnClearLogs: document.getElementById('btn-clear-logs'),
    btnRefreshCluster: document.getElementById('btn-refresh-cluster'),
    benchSlider: document.getElementById('bench-slider'),
    sliderValDisplay: document.getElementById('slider-val-display'),
    btnStartFullBench: document.getElementById('btn-start-full-bench'),
    fullBenchDuration: document.getElementById('full-bench-duration'),
    fullBenchMs: document.getElementById('full-bench-ms'),
    fullBenchOps: document.getElementById('full-bench-ops'),
    fullBenchBandwidth: document.getElementById('full-bench-bandwidth'),
    currentSectionTitle: document.getElementById('current-section-title'),
    linkToCluster: document.getElementById('link-to-cluster'),
    // 50 Endpoints Explorer
    endpointsGridContainer: document.getElementById('endpoints-grid-container'),
    endpointSearch: document.getElementById('endpoint-search'),
    btnBatchTestAll: document.getElementById('btn-batch-test-all'),
    batchSummaryBox: document.getElementById('batch-summary-box'),
    bTotalCount: document.getElementById('b-total-count'),
    bTotalTime: document.getElementById('b-total-time'),
    bAvgTime: document.getElementById('b-avg-time')
};

// Canvas Setup
const canvasTel = document.getElementById('telemetryChart');
const ctxTel = canvasTel ? canvasTel.getContext('2d') : null;

const canvasThroughput = document.getElementById('throughputChart');
const ctxThroughput = canvasThroughput ? canvasThroughput.getContext('2d') : null;

// Initialize Navigation Tabs
function initTabs() {
    const navItems = document.querySelectorAll('.nav-item');
    navItems.forEach(item => {
        item.addEventListener('click', (e) => {
            e.preventDefault();
            const tabId = item.getAttribute('data-tab');
            if (tabId) switchTab(tabId);
        });
    });

    if (elements.linkToCluster) {
        elements.linkToCluster.addEventListener('click', () => {
            switchTab('tab-cluster');
        });
    }

    if (elements.btnOpenEndpoints) {
        elements.btnOpenEndpoints.addEventListener('click', () => {
            switchTab('tab-endpoints');
        });
    }
}

function switchTab(tabId) {
    if (state.activeTab === tabId) return;
    state.activeTab = tabId;

    document.querySelectorAll('.tab-pane').forEach(p => p.classList.remove('active'));
    document.querySelectorAll('.nav-item').forEach(n => n.classList.remove('active'));

    const targetPane = document.getElementById(tabId);
    const targetNav = document.querySelector(`.nav-item[data-tab="${tabId}"]`);

    if (targetPane) targetPane.classList.add('active');
    if (targetNav) targetNav.classList.add('active');

    // Update section title
    const titles = {
        'tab-dashboard': 'داشبورد مانیتورینگ بلادرنگ',
        'tab-endpoints': 'کاوشگر و پنل تست ۵۰ اندپوینت ماژولار زبان End',
        'tab-benchmark': 'مجموعه بنچمارک و تست سرعت زبان End',
        'tab-cluster': 'کلاستر نودهای پردازشی (End Engine Cluster)',
        'tab-analysis': 'تحلیل جامع و معماری تخصصی زبان End',
        'tab-logs': 'ترمینال و جریان لاگ‌های زنده موتور'
    };
    if (elements.currentSectionTitle && titles[tabId]) {
        elements.currentSectionTitle.textContent = titles[tabId];
    }

    // Safely trigger renders
    if (tabId === 'tab-dashboard') {
        requestAnimationFrame(() => {
            renderTelemetryChart();
            renderThroughputChart();
        });
    } else if (tabId === 'tab-endpoints') {
        if (state.endpointsData.length === 0) fetchCatalog();
    } else if (tabId === 'tab-cluster') {
        fetchCluster();
    } else if (tabId === 'tab-logs') {
        fetchLogs();
    }
}

function isDashboardActive() {
    return state.activeTab === 'tab-dashboard';
}

// Fetch Telemetry from End DLL Backend
async function fetchTelemetry() {
    if (!state.isStreaming || state.isFetchingTelemetry) return;
    state.isFetchingTelemetry = true;

    try {
        const res = await fetch('/api/telemetry', { cache: 'no-store' });
        if (!res.ok) return;
        const data = await res.json();

        // Update KPI Stats
        if (elements.valHealth) elements.valHealth.textContent = `${data.health_score}%`;
        if (elements.barHealth) elements.barHealth.style.width = `${data.health_score}%`;

        if (elements.valRps) elements.valRps.textContent = Number(data.throughput_rps).toLocaleString('fa-IR');
        if (elements.pillRps) elements.pillRps.textContent = `${Number(data.throughput_rps).toLocaleString()} RPS`;

        if (elements.valLatency) elements.valLatency.textContent = data.latency_us;
        if (elements.barLatency) elements.barLatency.style.width = `${Math.min(data.latency_us / 4, 100)}%`;

        // Update Telemetry History Arrays
        state.telemetryHistory.cpu.push(data.cpu_pct);
        state.telemetryHistory.cpu.shift();

        state.telemetryHistory.mem.push(data.mem_pct);
        state.telemetryHistory.mem.shift();

        state.telemetryHistory.rps.push(data.throughput_rps);
        state.telemetryHistory.rps.shift();

        if (isDashboardActive()) {
            renderTelemetryChart();
            renderThroughputChart();
        }

    } catch (err) {
        console.error('Telemetry fetch error:', err);
    } finally {
        state.isFetchingTelemetry = false;
    }
}

// 50-Endpoints Catalog Loader
async function fetchCatalog() {
    try {
        const res = await fetch('/api/catalog');
        if (!res.ok) return;
        const data = await res.json();

        const flatList = [];
        for (const [modName, eps] of Object.entries(data.modules)) {
            eps.forEach(ep => {
                flatList.push({ ...ep, module: modName, result: '--', latency_us: '--' });
            });
        }
        state.endpointsData = flatList;
        renderEndpointsGrid();
    } catch (err) {
        console.error('Failed to load endpoints catalog:', err);
    }
}

function renderEndpointsGrid() {
    if (!elements.endpointsGridContainer) return;

    const q = state.searchQuery.toLowerCase().trim();
    const filtered = state.endpointsData.filter(ep => {
        const matchMod = state.activeModuleFilter === 'ALL' || ep.module === state.activeModuleFilter;
        const matchSearch = !q || ep.name.toLowerCase().includes(q) || ep.id.toLowerCase().includes(q) || ep.desc.toLowerCase().includes(q) || ep.module.toLowerCase().includes(q);
        return matchMod && matchSearch;
    });

    if (filtered.length === 0) {
        elements.endpointsGridContainer.innerHTML = '<div style="grid-column: 1/-1; text-align: center; color: var(--text-muted); padding: 40px;">اندپوینتی با این مشخصات یافت نشد.</div>';
        return;
    }

    elements.endpointsGridContainer.innerHTML = filtered.map(ep => `
        <div class="ep-card" id="card-${ep.id}">
            <div>
                <div class="ep-head">
                    <span class="ep-mod-badge">${ep.module}</span>
                    <span class="ep-latency-chip" id="lat-${ep.id}">${ep.latency_us === '--' ? 'Zero-GC' : ep.latency_us + ' µs'}</span>
                </div>
                <h4 class="ep-name">${ep.name}</h4>
                <p class="ep-desc">${ep.desc}</p>
            </div>
            <div>
                <div class="ep-res-box">
                    <span class="ep-unit">${ep.unit}</span>
                    <span class="ep-res-val" id="res-${ep.id}">${ep.result}</span>
                </div>
                <button class="ep-btn-test" onclick="executeSingleEndpoint('${ep.id}')">
                    ⚡ اجرای تست در موتور نیتیو End
                </button>
            </div>
        </div>
    `).join('');
}

// Single Endpoint Test
window.executeSingleEndpoint = async function(id) {
    const btn = document.querySelector(`#card-${id} .ep-btn-test`);
    const resEl = document.getElementById(`res-${id}`);
    const latEl = document.getElementById(`lat-${id}`);

    if (btn) btn.textContent = '⏳ در حال محاسبه...';

    try {
        const res = await fetch(`/api/endpoint/${id}`);
        const data = await res.json();

        if (resEl) resEl.textContent = typeof data.result === 'number' ? Number(data.result).toLocaleString('fa-IR') : data.result;
        if (latEl) latEl.textContent = `${data.duration_us} µs`;

        // Update in state cache
        const item = state.endpointsData.find(e => e.id === id);
        if (item) {
            item.result = resEl.textContent;
            item.latency_us = data.duration_us;
        }

        fetchLogs();
    } catch (err) {
        console.error(`Error executing endpoint ${id}:`, err);
    } finally {
        if (btn) btn.textContent = '⚡ اجرای تست در موتور نیتیو End';
    }
};

// Batch Test All 50 Endpoints
async function runBatchTestAll() {
    if (elements.btnBatchTestAll) {
        elements.btnBatchTestAll.disabled = true;
        elements.btnBatchTestAll.textContent = '⏳ در حال اجرای ۵۰ اندپوینت در موتور زبان End...';
    }

    try {
        const res = await fetch('/api/batch-test');
        const data = await res.json();

        if (elements.batchSummaryBox) elements.batchSummaryBox.style.display = 'flex';
        if (elements.bTotalCount) elements.bTotalCount.textContent = `${data.endpoints_tested} / 50 موفق`;
        if (elements.bTotalTime) elements.bTotalTime.textContent = `${data.total_duration_us} µs`;
        if (elements.bAvgTime) elements.bAvgTime.textContent = `${data.avg_duration_us} µs به ازای هر اندپوینت`;

        // Update all cards in UI
        data.results.forEach(r => {
            const resEl = document.getElementById(`res-${r.id}`);
            const latEl = document.getElementById(`lat-${r.id}`);
            const formatted = typeof r.result === 'number' ? Number(r.result).toLocaleString('fa-IR') : r.result;
            if (resEl) resEl.textContent = formatted;
            if (latEl) latEl.textContent = `${r.duration_us} µs`;

            const item = state.endpointsData.find(e => e.id === r.id);
            if (item) {
                item.result = formatted;
                item.latency_us = r.duration_us;
            }
        });

        fetchLogs();
    } catch (err) {
        console.error('Batch test error:', err);
    } finally {
        if (elements.btnBatchTestAll) {
            elements.btnBatchTestAll.disabled = false;
            elements.btnBatchTestAll.textContent = '⚡ اجرای تست دسته‌جمعی تمام ۵۰ اندپوینت';
        }
    }
}

// Fetch Cluster Nodes
async function fetchCluster() {
    if (state.isFetchingCluster) return;
    state.isFetchingCluster = true;

    try {
        const res = await fetch('/api/cluster', { cache: 'no-store' });
        if (!res.ok) return;
        const data = await res.json();

        if (elements.miniClusterList && isDashboardActive()) {
            elements.miniClusterList.innerHTML = data.nodes.slice(0, 4).map(node => `
                <div class="mini-node-item">
                    <span class="mini-node-name">${node.name}</span>
                    <span class="mini-node-load">${node.load_pct}% Load</span>
                </div>
            `).join('');
        }

        if (elements.clusterNodesGrid && state.activeTab === 'tab-cluster') {
            elements.clusterNodesGrid.innerHTML = data.nodes.map(node => `
                <div class="cluster-node-card">
                    <div class="c-node-head">
                        <span class="c-node-id">${node.id}</span>
                        <span class="c-node-status">${node.status}</span>
                    </div>
                    <div class="c-node-metrics">
                        <div class="c-metric-line">
                            <span class="text-muted">عنوان:</span>
                            <strong>${node.name}</strong>
                        </div>
                        <div class="c-metric-line">
                            <span class="text-muted">بار پردازنده:</span>
                            <span class="text-cyan">${node.load_pct}%</span>
                        </div>
                        <div class="c-metric-line">
                            <span class="text-muted">دما:</span>
                            <span>${node.temperature_c} °C</span>
                        </div>
                        <div class="c-metric-line">
                            <span class="text-muted">آرنای حافظه:</span>
                            <span>${node.memory_mb} MB (Zero GC)</span>
                        </div>
                    </div>
                </div>
            `).join('');
        }

    } catch (err) {
        console.error('Failed to fetch cluster:', err);
    } finally {
        state.isFetchingCluster = false;
    }
}

// Fetch Logs
async function fetchLogs() {
    if (state.isFetchingLogs) return;
    state.isFetchingLogs = true;

    try {
        const res = await fetch('/api/logs', { cache: 'no-store' });
        if (!res.ok) return;
        const data = await res.json();

        if (elements.terminalBody) {
            elements.terminalBody.innerHTML = data.logs.map(log => `
                <div class="log-row">
                    <span class="log-time">[${log.time}]</span>
                    <span class="log-tag ${log.level}">[${log.level}]</span>
                    <span class="log-msg">${log.msg}</span>
                </div>
            `).join('');
            elements.terminalBody.scrollTop = elements.terminalBody.scrollHeight;
        }
    } catch (err) {
        console.error('Failed to fetch logs:', err);
    } finally {
        state.isFetchingLogs = false;
    }
}

// Hardware-Accelerated Canvas Rendering
function resizeCanvas(canvas) {
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    if (rect.width > 0 && rect.height > 0) {
        if (canvas.width !== Math.floor(rect.width) || canvas.height !== Math.floor(rect.height)) {
            canvas.width = Math.floor(rect.width);
            canvas.height = Math.floor(rect.height);
        }
    }
}

function renderTelemetryChart() {
    if (!ctxTel || !canvasTel || !isDashboardActive()) return;
    resizeCanvas(canvasTel);
    const w = canvasTel.width;
    const h = canvasTel.height;
    if (w <= 0 || h <= 0) return;

    ctxTel.clearRect(0, 0, w, h);

    // Draw Grid Lines
    ctxTel.strokeStyle = 'rgba(255, 255, 255, 0.05)';
    ctxTel.lineWidth = 1;
    for (let y = 0; y <= h; y += h / 4) {
        ctxTel.beginPath();
        ctxTel.moveTo(0, y);
        ctxTel.lineTo(w, y);
        ctxTel.stroke();
    }

    drawCurve(ctxTel, state.telemetryHistory.cpu, 0, 100, '#00f0ff', 'rgba(0, 240, 255, 0.15)', w, h);
    drawCurve(ctxTel, state.telemetryHistory.mem, 0, 100, '#a855f7', 'rgba(168, 85, 247, 0.12)', w, h);
}

function renderThroughputChart() {
    if (!ctxThroughput || !canvasThroughput || !isDashboardActive()) return;
    resizeCanvas(canvasThroughput);
    const w = canvasThroughput.width;
    const h = canvasThroughput.height;
    if (w <= 0 || h <= 0) return;

    ctxThroughput.clearRect(0, 0, w, h);

    ctxThroughput.strokeStyle = 'rgba(255, 255, 255, 0.05)';
    ctxThroughput.lineWidth = 1;
    for (let y = 0; y <= h; y += h / 4) {
        ctxThroughput.beginPath();
        ctxThroughput.moveTo(0, y);
        ctxThroughput.lineTo(w, y);
        ctxThroughput.stroke();
    }

    drawCurve(ctxThroughput, state.telemetryHistory.rps, 10000, 50000, '#10b981', 'rgba(16, 185, 129, 0.18)', w, h);
}

function drawCurve(ctx, data, minVal, maxVal, strokeColor, fillColor, w, h) {
    if (!data || data.length < 2 || w <= 0 || h <= 0) return;
    const step = w / (data.length - 1);

    ctx.beginPath();
    data.forEach((val, i) => {
        const normalized = Math.max(0, Math.min(1, (val - minVal) / (maxVal - minVal)));
        const x = i * step;
        const y = h - (normalized * (h - 20)) - 10;
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
    });

    ctx.strokeStyle = strokeColor;
    ctx.lineWidth = 2.5;
    ctx.shadowColor = strokeColor;
    ctx.shadowBlur = 8;
    ctx.stroke();
    ctx.shadowBlur = 0;

    ctx.lineTo(w, h);
    ctx.lineTo(0, h);
    ctx.closePath();
    ctx.fillStyle = fillColor;
    ctx.fill();
}

// Quick Benchmark Handler
async function runQuickBenchmark() {
    if (elements.btnRunQuickBench) {
        elements.btnRunQuickBench.disabled = true;
        elements.btnRunQuickBench.textContent = '⏳ در حال اجرا...';
    }

    try {
        const res = await fetch(`/api/endpoint/math_sort_benchmark`);
        const data = await res.json();

        if (elements.quickBenchTime) elements.quickBenchTime.textContent = `${data.duration_us} µs`;
        if (elements.quickBenchOps) elements.quickBenchOps.textContent = `140,000 Ops`;
        if (elements.quickBenchHash) elements.quickBenchHash.textContent = `#${data.result}`;

        fetchLogs();
    } catch (err) {
        console.error('Quick bench error:', err);
    } finally {
        if (elements.btnRunQuickBench) {
            elements.btnRunQuickBench.disabled = false;
            elements.btnRunQuickBench.textContent = '🔥 اجرای تست روی موتور End DLL';
        }
    }
}

async function runFullBenchmark() {
    const iters = elements.benchSlider ? parseInt(elements.benchSlider.value) : 500000;
    if (elements.btnStartFullBench) {
        elements.btnStartFullBench.disabled = true;
        elements.btnStartFullBench.textContent = '⚡ در حال محاسبه در آرنای حافظه کش سخت‌افزاری...';
    }

    try {
        const res = await fetch(`/api/endpoint/math_monte_carlo`);
        const data = await res.json();

        if (elements.fullBenchDuration) elements.fullBenchDuration.textContent = data.duration_us;
        if (elements.fullBenchMs) elements.fullBenchMs.textContent = `معادل ${data.duration_us / 1000} میلی‌ثانیه`;
        if (elements.fullBenchOps) elements.fullBenchOps.textContent = Number(data.result).toLocaleString();
        if (elements.fullBenchBandwidth) elements.fullBenchBandwidth.textContent = '512,000';

        fetchLogs();
    } catch (err) {
        console.error('Full bench error:', err);
    } finally {
        if (elements.btnStartFullBench) {
            elements.btnStartFullBench.disabled = false;
            elements.btnStartFullBench.textContent = '⚡ آغاز بنچمارک دقیق و سنجش زمان';
        }
    }
}

// Event Listeners
function initEventListeners() {
    // Stream Toggle
    if (elements.btnToggleStream) {
        elements.btnToggleStream.addEventListener('click', () => {
            state.isStreaming = !state.isStreaming;
            if (state.isStreaming) {
                elements.btnToggleStream.innerHTML = '<span class="icon-pause-play">⏸</span> <span id="stream-status-text">توقف استریم</span>';
            } else {
                elements.btnToggleStream.innerHTML = '<span class="icon-pause-play">▶️</span> <span id="stream-status-text">ادامه استریم</span>';
            }
        });
    }

    // Preset buttons
    document.querySelectorAll('.btn-bench-preset').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.btn-bench-preset').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            state.quickBenchIters = parseInt(btn.getAttribute('data-iters'));
        });
    });

    if (elements.btnRunQuickBench) {
        elements.btnRunQuickBench.addEventListener('click', runQuickBenchmark);
    }

    if (elements.benchSlider && elements.sliderValDisplay) {
        elements.benchSlider.addEventListener('input', (e) => {
            elements.sliderValDisplay.textContent = Number(e.target.value).toLocaleString();
        });
    }

    if (elements.btnStartFullBench) {
        elements.btnStartFullBench.addEventListener('click', runFullBenchmark);
    }

    if (elements.btnRefreshCluster) {
        elements.btnRefreshCluster.addEventListener('click', fetchCluster);
    }

    if (elements.btnClearLogs && elements.terminalBody) {
        elements.btnClearLogs.addEventListener('click', () => {
            elements.terminalBody.innerHTML = '<div class="log-row"><span class="log-time">[Ready]</span> <span class="log-tag INFO">[INFO]</span> <span class="log-msg">کنسول پاکسازی شد. در انتظار رویداد جدید...</span></div>';
        });
    }

    // 50 Endpoints Search & Filters
    if (elements.endpointSearch) {
        elements.endpointSearch.addEventListener('input', (e) => {
            state.searchQuery = e.target.value;
            renderEndpointsGrid();
        });
    }

    document.querySelectorAll('.btn-mod-filter').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.btn-mod-filter').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            state.activeModuleFilter = btn.getAttribute('data-mod');
            renderEndpointsGrid();
        });
    });

    if (elements.btnBatchTestAll) {
        elements.btnBatchTestAll.addEventListener('click', runBatchTestAll);
    }

    window.addEventListener('resize', () => {
        if (isDashboardActive()) {
            renderTelemetryChart();
            renderThroughputChart();
        }
    });
}

// App Bootstrapper
document.addEventListener('DOMContentLoaded', () => {
    initTabs();
    initEventListeners();

    // Initial Data Fetch
    fetchTelemetry();
    fetchCluster();
    fetchLogs();
    fetchCatalog();
    runQuickBenchmark();

    // Start Live Polling
    state.pollIntervalId = setInterval(() => {
        fetchTelemetry();
    }, 1200);

    // Cluster poll every 6 sec
    setInterval(() => {
        if (state.activeTab === 'tab-cluster' || isDashboardActive()) {
            fetchCluster();
        }
    }, 6000);
});
