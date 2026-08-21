const btnBuy = document.getElementById('btn-buy');
const btnSell = document.getElementById('btn-sell');
const tradeLog = document.getElementById('trade-log');
const valBlockHeight = document.getElementById('val-block-height');
const valMerkle = document.getElementById('val-merkle');

async function executeTrade(side) {
    tradeLog.textContent = `در حال ارسال پکت سفارش ${side} به موتور تطبیق در حافظه رم...`;
    try {
        const res = await fetch('/api/ledger/order');
        const data = await res.json();

        tradeLog.textContent = `[MATCHED TRADE #${data.order_id}]
✅ سفارش با موفقیت در موتور End تطبیق داده شد:
نوع: ${side}
جفت‌ارز: ${data.symbol}
قیمت معامله: $${data.price.toLocaleString()}
حجم: ${data.quantity} BTC
ارزش کل: $${data.total_usd.toLocaleString()}
تاخیر موتور تطبیق: ${data.matching_latency_us} µs
وضعیت کش: ${data.redis_cache}
ثبت در دیتابیس: ${data.postgres_wal}
هش مرکل بلاک: ${data.block_merkle_hash}`;

        // Update block info
        fetchBlocks();
    } catch (e) {
        tradeLog.textContent = 'خطا در ثبت سفارش: ' + e;
    }
}

async function fetchBlocks() {
    try {
        const res = await fetch('/api/ledger/blocks');
        const data = await res.json();
        valBlockHeight.textContent = `#${data.ledger_height.toLocaleString()}`;
        valMerkle.textContent = data.merkle_root;
    } catch (e) {
        console.warn('Blocks Error:', e);
    }
}

btnBuy.addEventListener('click', () => executeTrade('خرید (BUY)'));
btnSell.addEventListener('click', () => executeTrade('فروش (SELL)'));

fetchBlocks();
