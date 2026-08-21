const canvas = document.getElementById('game-canvas');
const ctx = canvas.getContext('2d');
const btnBurst = document.getElementById('btn-burst');
const btnRaft = document.getElementById('btn-raft');
const valEnergy = document.getElementById('val-energy');
const valTerm = document.getElementById('val-term');
const valFrameTime = document.getElementById('val-frame-time');
const valRaftStatus = document.getElementById('val-raft-status');

let particles = [];
const PARTICLE_COUNT = 300;

for (let i = 0; i < PARTICLE_COUNT; i++) {
    particles.push({
        x: Math.random() * canvas.width,
        y: Math.random() * canvas.height,
        vx: (Math.random() - 0.5) * 4,
        vy: (Math.random() - 0.5) * 4,
        color: ['#a855f7', '#06b6d4', '#ec4899', '#3b82f6'][Math.floor(Math.random() * 4)],
        size: Math.random() * 2.5 + 1
    });
}

function renderLoop() {
    ctx.fillStyle = 'rgba(5, 4, 10, 0.25)';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    for (let p of particles) {
        p.x += p.vx;
        p.y += p.vy;

        if (p.x <= 0 || p.x >= canvas.width) p.vx *= -1;
        if (p.y <= 0 || p.y >= canvas.height) p.vy *= -1;

        ctx.beginPath();
        ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
        ctx.fillStyle = p.color;
        ctx.shadowColor = p.color;
        ctx.shadowBlur = 8;
        ctx.fill();
    }

    requestAnimationFrame(renderLoop);
}
requestAnimationFrame(renderLoop);

btnBurst.addEventListener('click', async () => {
    try {
        const res = await fetch('/api/game/tick');
        const data = await res.json();
        
        valEnergy.textContent = `${data.total_kinetic_energy.toLocaleString()} J`;
        valFrameTime.textContent = `${data.frame_time_us} µs`;

        // Explode particles from center
        for (let p of particles) {
            p.vx = (Math.random() - 0.5) * 12;
            p.vy = (Math.random() - 0.5) * 12;
        }
    } catch (e) {
        console.warn('Tick Error:', e);
    }
});

btnRaft.addEventListener('click', async () => {
    try {
        const res = await fetch('/api/game/raft');
        const data = await res.json();
        
        valTerm.textContent = `Term ${data.term}`;
        valRaftStatus.textContent = `👑 Node ${data.leader_node_id} (Quorum ${data.quorum_nodes} Synced)`;
    } catch (e) {
        console.warn('Raft Error:', e);
    }
});
