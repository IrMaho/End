const btnCheckout = document.getElementById('btn-checkout');
const checkoutOutput = document.getElementById('checkout-output');

btnCheckout.addEventListener('click', async () => {
    checkoutOutput.textContent = 'در حال ارسال درخواست به خط لوله میدلورها در بک‌اند End...';
    try {
        const res = await fetch('/api/shop/checkout');
        const data = await res.json();

        checkoutOutput.textContent = `[ORDER CHECKOUT SUCCESS]
✅ سفارش با موفقیت در لایه تمیز UseCase پردازش شد:
شناسه سفارش: #${data.order_id}
محصول: ${data.item}
تعداد: ${data.quantity} عدد (فی $${data.unit_price})
هزینه ارسال: $${data.shipping_usd}
مبلغ کل کسرشده: $${data.total_usd}.00

دیزاین‌پترن‌های اجراشده در حافظه رم:
• ${data.patterns_used.join('\n• ')}

مراحل خط لوله (Pipeline Stages):
${data.pipeline_stages.map((s, i) => `  ${i + 1}. ${s} (Passed ✔)`).join('\n')}

وضعیت انطباق معماری: ${data.architecture}`;
    } catch (e) {
        checkoutOutput.textContent = 'خطا در ثبت سفارش: ' + e;
    }
});
