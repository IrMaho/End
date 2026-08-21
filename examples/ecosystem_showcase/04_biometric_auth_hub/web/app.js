const btnPasskey = document.getElementById('btn-passkey');
const btnOauth = document.getElementById('btn-oauth');
const btnArgon = document.getElementById('btn-argon');
const authOutput = document.getElementById('auth-output');

btnPasskey.addEventListener('click', async () => {
    authOutput.textContent = 'در حال ارتباط با سنسور بیومتریک و اعتبارسنجی امضای ECDSA در هسته End...';
    try {
        const res = await fetch('/api/auth/passkey');
        const data = await res.json();

        authOutput.textContent = `[PASSKEY AUTHENTICATION SUCCESS]
✅ هویت کاربر با موفقیت تایید شد:
پروتکل: ${data.protocol}
شناسه کاربری: ${data.user_id}
دستگاه احراز هویت: ${data.authenticator}
اعتبارسنجی امضای سخت‌افزاری: ${data.signature_verified ? 'تایید شد (P-256 Valid)' : 'نامعتبر'}
امتیاز اعتماد صفر-تراست: ${data.zero_trust_score}%
نوع توکن صادرشده: ${data.token_type}`;
    } catch (e) {
        authOutput.textContent = 'خطا در احراز هویت Passkey: ' + e;
    }
});

btnOauth.addEventListener('click', async () => {
    authOutput.textContent = 'در حال تولید چالش امنیتی PKCE S256 و تبادل کد...';
    try {
        const res = await fetch('/api/auth/oauth');
        const data = await res.json();

        authOutput.textContent = `[OAUTH2 PKCE AUTHORIZATION CODE]
✅ جریان کد امنیتی کامل شد:
پروتکل: ${data.flow}
ارائه‌دهنده هویت: ${data.provider}
دسترسی‌ها: ${data.scope}
مدت اعتبار توکن: ${data.expires_in} ثانیه (1 ساعت)`;
    } catch (e) {
        authOutput.textContent = 'خطا در جریان OAuth2: ' + e;
    }
});

btnArgon.addEventListener('click', async () => {
    authOutput.textContent = 'در حال اجرای هشینگ حافظه‌محور و مقاوم در برابر کرک GPU (Argon2id)...';
    try {
        const res = await fetch('/api/auth/argon2');
        const data = await res.json();

        authOutput.textContent = `[ARGON2ID CRYPTOGRAPHIC VERIFICATION]
✅ هش پسورد با موفقیت بررسی و تایید شد:
الگوریتم: ${data.algorithm}
مصرف حافظه رم: ${data.memory_cost_kib / 1024} MB RAM
تعداد تکرار زمانی: ${data.time_cost_iterations} Iterations
مقاومت سخت‌افزاری: ${data.gpu_resistance}
زمان محاسبات: ${data.verification_latency_ms} ms`;
    } catch (e) {
        authOutput.textContent = 'خطا در Argon2id: ' + e;
    }
});
