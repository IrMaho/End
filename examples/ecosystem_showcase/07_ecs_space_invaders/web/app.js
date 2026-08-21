const canvas = document.getElementById('space-canvas');
const ctx = canvas.getContext('2d');
const btnWave = document.getElementById('btn-wave');
const btnFire = document.getElementById('btn-fire');
const valWave = document.getElementById('val-wave');
const valScore = document.getElementById('val-score');
const valFsm = document.getElementById('val-fsm');

let aliens = [];
let lasers = [];
let shipX = canvas.width / 2;

function initAliens() {
    aliens = [];
    for (let r = 0; r < 4; r++) {
        for (let c = 0; c < 8; c++) {
            aliens.push({
                x: 100 + c * 75,
                y: 50 + r * 45,
                alive: true,
                color: ['#f43f5e', '#fb7185', '#06b6d4', '#38bdf8'][r]
            });
        }
    }
}
initAliens();

function renderGame() {
    ctx.fillStyle = 'rgba(3, 2, 6, 0.3)';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Player Ship
    ctx.fillStyle = '#06b6d4';
    ctx.beginPath();
    ctx.moveTo(shipX, canvas.height - 35);
    ctx.lineTo(shipX - 18, canvas.height - 10);
    ctx.lineTo(shipX + 18, canvas.height - 10);
    ctx.closePath();
    ctx.fill();

    // Lasers
    ctx.fillStyle = '#f43f5e';
    for (let l of lasers) {
        l.y -= 8;
        ctx.fillRect(l.x - 2, l.y, 4, 12);

        // Check collision with aliens
        for (let a of aliens) {
            if (a.alive && Math.abs(l.x - a.x) < 20 && Math.abs(l.y - a.y) < 15) {
                a.alive = false;
                l.y = -100;
            }
        }
    }

    // Aliens
    for (let a of aliens) {
        if (a.alive) {
            ctx.fillStyle = a.color;
            ctx.fillRect(a.x - 14, a.y - 10, 28, 20);
        }
    }

    requestAnimationFrame(renderGame);
}
requestAnimationFrame(renderGame);

btnFire.addEventListener('click', () => {
    lasers.push({ x: shipX, y: canvas.height - 35 });
});

canvas.addEventListener('mousemove', (e) => {
    const rect = canvas.getBoundingClientRect();
    shipX = (e.clientX - rect.left) * (canvas.width / rect.width);
});

btnWave.addEventListener('click', async () => {
    try {
        const res = await fetch('/api/game/spawn');
        const data = await res.json();

        valWave.textContent = `Wave ${data.wave}`;
        valScore.textContent = `${data.score} Pts`;
        valFsm.textContent = data.fsm_state;

        initAliens();
    } catch (e) {
        console.warn('Spawn error:', e);
    }
});
