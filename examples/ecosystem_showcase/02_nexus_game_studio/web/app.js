// ============================================================================
// 120 FPS CLIENT ENGINE & CANVAS VISUALIZER
// ============================================================================

const canvas = document.getElementById('game-canvas');
const ctx = canvas.getContext('2d');

let particles = [];
let gravityFlipped = false;
let turboMode = false;
let serverState = null;

// Initialize particles
for (let i = 0; i < 64; i++) {
    particles.push({
        x: 400 + (Math.random() - 0.5) * 300,
        y: 210 + (Math.random() - 0.5) * 200,
        vx: (Math.random() - 0.5) * 4,
        vy: (Math.random() - 0.5) * 4,
        radius: 3 + Math.random() * 4,
        color: i % 2 === 0 ? '#00f0ff' : '#ff007f'
    });
}

// Fetch live server physics state from EndNative server (Port 5050)
async function syncServerState() {
    try {
        const res = await fetch('/api/game/state');
        if (res.ok) {
            serverState = await res.json();
            updateUI(serverState);
        }
    } catch (e) {
        console.warn('Syncing with EndNexus...', e);
    }
}

function updateUI(data) {
    if (!data) return;
    const scoreEl = document.getElementById('player-score');
    const rankEl = document.getElementById('player-rank');
    const roomEl = document.getElementById('room-id');
    const pingEl = document.getElementById('room-ping');

    if (scoreEl) scoreEl.textContent = Number(data.player.score).toLocaleString('fa-IR');
    if (rankEl) rankEl.textContent = `Rank #${data.player.rank} (Master)`;
    if (roomEl) roomEl.textContent = `Arena #${data.matchmaker.room_id}`;
    if (pingEl) pingEl.textContent = `${data.matchmaker.ping_ms} ms`;
}

// 120 FPS Render Loop
let lastTime = performance.now();
let frames = 0;
let fpsTimer = performance.now();

function gameLoop(now) {
    frames++;
    if (now - fpsTimer >= 1000) {
        document.getElementById('canvas-fps').textContent = `${frames} FPS`;
        frames = 0;
        fpsTimer = now;
    }

    const dt = (now - lastTime) / 1000;
    lastTime = now;

    // Clear canvas
    ctx.fillStyle = 'rgba(2, 4, 8, 0.25)';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Draw central attractor
    ctx.beginPath();
    ctx.arc(canvas.width / 2, canvas.height / 2, 8, 0, Math.PI * 2);
    ctx.fillStyle = '#ffe600';
    ctx.shadowBlur = 20;
    ctx.shadowColor = '#ffe600';
    ctx.fill();
    ctx.shadowBlur = 0;

    // Update & draw particles
    const speedMultiplier = turboMode ? 2.5 : 1.0;
    const grav = (gravityFlipped ? -150 : 150) * speedMultiplier;

    particles.forEach((p, idx) => {
        const dx = (canvas.width / 2) - p.x;
        const dy = (canvas.height / 2) - p.y;
        const dist = Math.sqrt(dx * dx + dy * dy) + 10;
        
        // Gravitational force
        p.vx += (dx / dist) * (grav / dist) * dt;
        p.vy += (dy / dist) * (grav / dist) * dt;

        p.x += p.vx * speedMultiplier;
        p.y += p.vy * speedMultiplier;

        // Bounce walls
        if (p.x < p.radius || p.x > canvas.width - p.radius) p.vx *= -0.9;
        if (p.y < p.radius || p.y > canvas.height - p.radius) p.vy *= -0.9;

        // Draw particle
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2);
        ctx.fillStyle = p.color;
        ctx.shadowBlur = 12;
        ctx.shadowColor = p.color;
        ctx.fill();
        ctx.shadowBlur = 0;

        // Connect nearby particles
        for (let j = idx + 1; j < particles.length; j++) {
            const p2 = particles[j];
            const d = Math.hypot(p.x - p2.x, p.y - p2.y);
            if (d < 65) {
                ctx.beginPath();
                ctx.moveTo(p.x, p.y);
                ctx.lineTo(p2.x, p2.y);
                ctx.strokeStyle = `rgba(0, 240, 255, ${1 - (d / 65)})`;
                ctx.lineWidth = 0.8;
                ctx.stroke();
            }
        }
    });

    requestAnimationFrame(gameLoop);
}

// Interactive Controls
document.getElementById('btn-spawn').addEventListener('click', async () => {
    try {
        const res = await fetch('/api/game/spawn');
        const data = await res.json();
        for (let i = 0; i < 16; i++) {
            particles.push({
                x: 400,
                y: 210,
                vx: (Math.random() - 0.5) * 12,
                vy: (Math.random() - 0.5) * 12,
                radius: 4,
                color: '#ffe600'
            });
        }
    } catch (e) {
        console.error(e);
    }
});

document.getElementById('btn-gravity').addEventListener('click', (e) => {
    gravityFlipped = !gravityFlipped;
    e.target.textContent = gravityFlipped ? '🌀 گرانش: دافعه (Repel)' : '🌀 معکوس کردن گرانش فیزیک';
});

document.getElementById('btn-turbo').addEventListener('click', (e) => {
    turboMode = !turboMode;
    e.target.textContent = turboMode ? '⚡ حالت توربو: فعال (240 Hz)' : '⚡ حالت توربو (240 Hz)';
});

// Start loops
setInterval(syncServerState, 800);
requestAnimationFrame(gameLoop);
