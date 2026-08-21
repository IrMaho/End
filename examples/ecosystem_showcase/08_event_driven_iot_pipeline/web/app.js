const btnSample = document.getElementById('btn-sample');
const valTemp = document.getElementById('val-temp');
const barTemp = document.getElementById('bar-temp');
const valPressure = document.getElementById('val-pressure');
const barPressure = document.getElementById('bar-pressure');
const valAnomaly = document.getElementById('val-anomaly');
const barAnomaly = document.getElementById('bar-anomaly');
const iotOutput = document.getElementById('iot-output');
const fsmNodeDisplay = document.getElementById('fsm-node-display');

btnSample.addEventListener('click', async () => {
    iotOutput.textContent = 'در حال دریافت پکت سنسور صنعتی در خط لوله Pipeline...';
    try {
        const res = await fetch('/api/iot/telemetry');
        const data = await res.json();

        valTemp.textContent = `${data.temperature_c} °C`;
        barTemp.style.width = `${Math.min(100, data.temperature_c)}%`;

        valPressure.textContent = `${data.pressure_kpa} kPa`;
        barPressure.style.width = `${Math.min(100, (data.pressure_kpa / 1200) * 100)}%`;

        valAnomaly.textContent = data.anomaly_score > 0 ? `${data.anomaly_score} (CRITICAL ALERT)` : '0.00 (Normal)';
        barAnomaly.style.width = `${Math.min(100, (data.anomaly_score / 200) * 100)}%`;

        fsmNodeDisplay.textContent = `2. ${data.fsm_state}`;

        iotOutput.textContent = `[IOT EVENT PIPELINE PROCESSED]
✅ داده‌های سنسور با موفقیت توسط هسته End ممیزی شدند:
شناسه تجهیز: ${data.device_id}
دما: ${data.temperature_c} °C
فشار: ${data.pressure_kpa} kPa
امتیاز ناهنجاری: ${data.anomaly_score}
وضعیت ماشین حالت (FSM): ${data.fsm_state}
تاخیر کل خط لوله: ${data.pipeline_latency_us} µs

پترن‌های اعمال‌شده:
• ${data.patterns_applied.join('\n• ')}`;
    } catch (e) {
        iotOutput.textContent = 'خطا در ارتباط با SCADA: ' + e;
    }
});
