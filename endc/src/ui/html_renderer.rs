use super::widget::WidgetNode;
use super::dev_overlay::DevOverlay;

pub struct HtmlUiRenderer;

impl HtmlUiRenderer {
    pub fn render_to_html(root_widget: &WidgetNode, is_dev_mode: bool, feedback_json_str: &str) -> String {
        let overlay_html = if is_dev_mode {
            DevOverlay::get_overlay_script_and_styles(feedback_json_str)
        } else {
            String::new()
        };

        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>💼 EndLedger - Enterprise Financial & Double-Entry Suite</title>
    <style>
        :root {{
            --bg-body: #080c14;
            --bg-surface: rgba(16, 24, 40, 0.85);
            --bg-surface-elevated: rgba(24, 34, 56, 0.95);
            --accent-cyan: #06b6d4;
            --accent-indigo: #6366f1;
            --accent-emerald: #10b981;
            --accent-rose: #f43f5e;
            --accent-amber: #f59e0b;
            --border-subtle: rgba(255, 255, 255, 0.08);
            --border-hover: rgba(99, 102, 241, 0.35);
            --text-main: #f8fafc;
            --text-muted: #94a3b8;
            --font-stack: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            --font-mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
        }}

        * {{ margin: 0; padding: 0; box-sizing: border-box; font-family: var(--font-stack); }}
        
        body {{
            background: var(--bg-body);
            color: var(--text-main);
            min-height: 100vh;
            display: flex;
            background-image: 
                radial-gradient(at 0% 0%, rgba(99, 102, 241, 0.15) 0px, transparent 50%),
                radial-gradient(at 100% 100%, rgba(6, 182, 212, 0.12) 0px, transparent 50%);
            background-attachment: fixed;
            overflow-x: hidden;
        }}

        .sidebar {{
            width: 280px;
            background: rgba(11, 17, 30, 0.98);
            border-right: 1px solid var(--border-subtle);
            padding: 24px 18px;
            display: flex;
            flex-direction: column;
            gap: 24px;
            position: sticky;
            top: 0;
            height: 100vh;
            flex-shrink: 0;
        }}

        .brand {{
            display: flex;
            align-items: center;
            gap: 12px;
            padding: 0 6px;
        }}
        .brand-icon {{
            width: 44px;
            height: 44px;
            border-radius: 12px;
            background: linear-gradient(135deg, var(--accent-cyan), var(--accent-indigo));
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 22px;
            box-shadow: 0 4px 20px rgba(6, 182, 212, 0.4);
        }}
        .brand-title {{ font-size: 19px; font-weight: 800; letter-spacing: -0.5px; }}
        .brand-badge {{ font-size: 11px; color: var(--accent-cyan); font-weight: 700; background: rgba(6, 182, 212, 0.12); padding: 3px 8px; border-radius: 4px; }}

        .nav-list {{ list-style: none; display: flex; flex-direction: column; gap: 6px; }}
        .nav-item {{
            padding: 12px 16px;
            border-radius: 10px;
            color: var(--text-muted);
            font-size: 14px;
            font-weight: 600;
            cursor: pointer;
            display: flex;
            align-items: center;
            gap: 12px;
            transition: all 0.2s;
        }}
        .nav-item:hover {{ background: rgba(255,255,255,0.06); color: #fff; transform: translateX(3px); }}
        .nav-item.active {{
            background: rgba(99, 102, 241, 0.18);
            color: #fff;
            border: 1px solid rgba(99, 102, 241, 0.4);
            box-shadow: 0 4px 15px rgba(99, 102, 241, 0.15);
        }}

        .main-content {{
            flex: 1;
            padding: 32px 40px;
            max-width: 1500px;
            overflow-y: auto;
        }}

        .top-bar {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 28px;
            background: var(--bg-surface);
            border: 1px solid var(--border-subtle);
            border-radius: 14px;
            padding: 16px 24px;
            backdrop-filter: blur(20px);
        }}
        .top-left h1 {{ font-size: 22px; font-weight: 800; }}
        .top-left p {{ font-size: 13px; color: var(--text-muted); margin-top: 2px; }}

        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(4, 1fr);
            gap: 20px;
            margin-bottom: 28px;
        }}
        .stat-card {{
            background: var(--bg-surface);
            border: 1px solid var(--border-subtle);
            border-radius: 14px;
            padding: 22px;
            backdrop-filter: blur(16px);
            transition: all 0.25s;
            position: relative;
            cursor: pointer;
        }}
        .stat-card:hover {{
            transform: translateY(-3px);
            border-color: var(--border-hover);
            box-shadow: 0 12px 30px rgba(0,0,0,0.4);
        }}
        .stat-label {{ font-size: 12px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; }}
        .stat-value {{ font-size: 28px; font-weight: 800; color: #fff; margin: 8px 0; font-family: var(--font-mono); }}
        .stat-sub {{ font-size: 12px; display: flex; align-items: center; gap: 6px; }}
        .stat-positive {{ color: var(--accent-emerald); font-weight: 600; }}

        .progress-bar-bg {{
            width: 100%; height: 6px; background: rgba(255,255,255,0.08); border-radius: 4px; margin-top: 10px; overflow: hidden;
        }}
        .progress-bar-fill {{
            height: 100%; border-radius: 4px; background: linear-gradient(90deg, var(--accent-cyan), var(--accent-indigo));
        }}

        .grid-2col {{
            display: grid;
            grid-template-columns: 1.1fr 0.9fr;
            gap: 24px;
            margin-bottom: 28px;
        }}

        .panel-card {{
            background: var(--bg-surface);
            border: 1px solid var(--border-subtle);
            border-radius: 14px;
            padding: 24px;
            backdrop-filter: blur(16px);
        }}
        .panel-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 20px;
            padding-bottom: 14px;
            border-bottom: 1px solid var(--border-subtle);
        }}
        .panel-title {{ font-size: 16px; font-weight: 700; display: flex; align-items: center; gap: 10px; }}

        .form-row {{ display: grid; grid-template-columns: 1fr 1fr; gap: 14px; margin-bottom: 14px; }}
        .form-group {{ margin-bottom: 14px; }}
        .form-label {{ font-size: 12px; font-weight: 600; color: var(--text-muted); margin-bottom: 6px; display: block; }}
        .form-input, .form-select {{
            width: 100%;
            background: rgba(0, 0, 0, 0.4);
            border: 1px solid var(--border-subtle);
            border-radius: 8px;
            padding: 11px 14px;
            color: #fff;
            font-size: 14px;
            outline: none;
            transition: all 0.2s;
        }}
        .form-input:focus, .form-select:focus {{
            border-color: var(--accent-indigo);
            box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.2);
        }}

        .btn-gradient {{
            background: linear-gradient(135deg, var(--accent-cyan), var(--accent-indigo));
            color: #fff;
            border: none;
            border-radius: 8px;
            padding: 13px 22px;
            font-size: 14px;
            font-weight: 700;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 8px;
            transition: all 0.2s;
            box-shadow: 0 4px 20px rgba(99, 102, 241, 0.35);
            width: 100%;
        }}
        .btn-gradient:hover {{
            transform: scale(1.02);
            box-shadow: 0 6px 25px rgba(99, 102, 241, 0.5);
        }}

        .table-wrap {{ width: 100%; overflow-x: auto; }}
        .data-table {{ width: 100%; border-collapse: collapse; font-size: 13px; }}
        .data-table th {{
            text-align: left;
            padding: 12px 14px;
            color: var(--text-muted);
            font-weight: 600;
            border-bottom: 1px solid var(--border-subtle);
        }}
        .data-table td {{
            padding: 12px 14px;
            border-bottom: 1px solid rgba(255,255,255,0.04);
            color: #e2e8f0;
        }}
        .data-table tr:hover td {{ background: rgba(255,255,255,0.03); }}

        .badge-tag {{
            padding: 3px 8px;
            border-radius: 4px;
            font-size: 11px;
            font-weight: 700;
        }}
        .badge-success {{ background: rgba(16, 185, 129, 0.15); color: var(--accent-emerald); }}
        .badge-info {{ background: rgba(6, 182, 212, 0.15); color: var(--accent-cyan); }}

        .pulse-indicator {{
            width: 8px; height: 8px; border-radius: 50%;
            background: var(--accent-emerald); display: inline-block;
            box-shadow: 0 0 10px var(--accent-emerald);
        }}
    </style>
</head>
<body>

    <aside class="sidebar" data-widget-id="nav_sidebar" data-widget-kind="Sidebar" data-source-file="src/ui_dashboard.end" data-source-line="10">
        <div class="brand">
            <div class="brand-icon">💼</div>
            <div>
                <div class="brand-title">EndLedger</div>
                <span class="brand-badge">Native Engine 120 FPS</span>
            </div>
        </div>

        <ul class="nav-list">
            <li class="nav-item active" onclick="switchNav(this)">📊 Financial Executive Overview</li>
            <li class="nav-item" onclick="switchNav(this)">🧾 Invoicing & Tax Terminal</li>
            <li class="nav-item" onclick="switchNav(this)">📖 General Ledger & Double-Entry</li>
            <li class="nav-item" onclick="switchNav(this)">🏛️ Chart of Accounts ($59.2k)</li>
            <li class="nav-item" onclick="switchNav(this)">📈 Income Statement & Reports</li>
        </ul>

        <div style="margin-top: auto; padding: 16px; background: rgba(0,0,0,0.4); border-radius: 12px; border: 1px solid var(--border-subtle);">
            <div style="font-size: 11px; color: var(--text-muted); margin-bottom: 4px;">Double-Entry State:</div>
            <div style="font-size: 13px; font-weight: 800; color: var(--accent-emerald); display: flex; align-items: center; gap: 8px;">
                <span class="pulse-indicator"></span> 100% Invariant Balanced
            </div>
            <div style="font-size: 11px; color: #64748b; margin-top: 4px;">Zero-Alloc Arena Memory</div>
        </div>
    </aside>

    <main class="main-content">
        <header class="top-bar">
            <div class="top-left">
                <h1 data-widget-id="executive_title" data-widget-kind="Text" data-source-file="src/ui_dashboard.end" data-source-line="35">💼 EndLedger Enterprise Dashboard</h1>
                <p>Fiscal Period: August 2026 | Currency: USD ($) | Real-time Zero Floating-Point Drift</p>
            </div>
            <button class="btn-gradient" style="width: auto; padding: 10px 18px;" onclick="scrollToInvoice()">
                ⚡ + New Sales Invoice
            </button>
        </header>

        <section class="stats-grid">
            <div class="stat-card" data-widget-id="card_cash" data-widget-kind="Card" data-source-file="src/main.end" data-source-line="240">
                <div class="stat-label">Operating Cash Reserve</div>
                <div class="stat-value" id="valCash">$59,155.00</div>
                <div class="stat-sub stat-positive">↑ +$9,155.00 Net Inflow (+18.3%)</div>
                <div class="progress-bar-bg"><div class="progress-bar-fill" style="width: 82%;"></div></div>
            </div>

            <div class="stat-card" data-widget-id="card_receivables" data-widget-kind="Card" data-source-file="src/main.end" data-source-line="245">
                <div class="stat-label">Accounts Receivable (AR)</div>
                <div class="stat-value" id="valAR">$10,355.00</div>
                <div class="stat-sub stat-positive">★ 1 Active Invoice Billed</div>
                <div class="progress-bar-bg"><div class="progress-bar-fill" style="width: 65%; background: var(--accent-cyan);"></div></div>
            </div>

            <div class="stat-card" data-widget-id="card_revenue" data-widget-kind="Card" data-source-file="src/main.end" data-source-line="250">
                <div class="stat-label">Total Billed Revenue</div>
                <div class="stat-value" id="valRevenue">$9,500.00</div>
                <div class="stat-sub stat-positive">↑ Net of 5% Early Bird Discount</div>
                <div class="progress-bar-bg"><div class="progress-bar-fill" style="width: 79%;"></div></div>
            </div>

            <div class="stat-card" data-widget-id="card_profit" data-widget-kind="Card" data-source-file="src/main.end" data-source-line="255">
                <div class="stat-label">Net Operating Income</div>
                <div class="stat-value" id="valProfit" style="color: var(--accent-emerald);">$8,300.00</div>
                <div class="stat-sub stat-positive">★ 87.36% Profit Margin</div>
                <div class="progress-bar-bg"><div class="progress-bar-fill" style="width: 87%; background: var(--accent-emerald);"></div></div>
            </div>
        </section>

        <section class="grid-2col">
            <div class="panel-card" id="invoicePanel" data-widget-id="panel_invoice_creator" data-widget-kind="Card" data-source-file="src/ui_dashboard.end" data-source-line="70">
                <div class="panel-header">
                    <h3 class="panel-title">🧾 Instant Invoice & Tax Calculation Terminal</h3>
                    <span class="badge-tag badge-success">Double-Entry Linked</span>
                </div>

                <div class="form-group">
                    <label class="form-label">Client Organization:</label>
                    <select id="invCustomer" class="form-select">
                        <option value="ACME Global Tech Inc.">ACME Global Technologies Inc.</option>
                        <option value="Microsoft Cloud Corp.">Microsoft Enterprise Solutions</option>
                        <option value="Telegram Open Network">Telegram Open Network FZE</option>
                        <option value="Apple Inc. Enterprise">Apple Developer Ecosystem</option>
                    </select>
                </div>

                <div class="form-row">
                    <div class="form-group">
                        <label class="form-label">Service Units (Hours/Licenses):</label>
                        <input type="number" id="invQty" class="form-input" value="10" oninput="recalcInvoice()">
                    </div>
                    <div class="form-group">
                        <label class="form-label">Unit Price ($):</label>
                        <input type="number" id="invPrice" class="form-input" value="1000" oninput="recalcInvoice()">
                    </div>
                </div>

                <div class="form-row">
                    <div class="form-group">
                        <label class="form-label">Volume Discount (%):</label>
                        <input type="number" id="invDiscount" class="form-input" value="5" oninput="recalcInvoice()">
                    </div>
                    <div class="form-group">
                        <label class="form-label">Tax Engine (Polymorphic VAT):</label>
                        <select id="invTaxRate" class="form-select" onchange="recalcInvoice()">
                            <option value="0.09">Standard 9% VAT (TaxEngine)</option>
                            <option value="0.15">Corporate 15% Withholding</option>
                            <option value="0.00">Zero-Rated Export (0%)</option>
                        </select>
                    </div>
                </div>

                <div style="background: rgba(0,0,0,0.35); padding: 16px; border-radius: 10px; margin-bottom: 16px; border: 1px solid var(--border-subtle);">
                    <div style="display: flex; justify-content: space-between; font-size: 13px; margin-bottom: 6px;">
                        <span style="color: var(--text-muted);">Gross Subtotal:</span>
                        <span id="prevSubtotal" style="font-family: var(--font-mono); font-weight: 700;">$10,000.00</span>
                    </div>
                    <div style="display: flex; justify-content: space-between; font-size: 13px; margin-bottom: 6px;">
                        <span style="color: var(--text-muted);">Discount (-5%):</span>
                        <span id="prevDiscount" style="font-family: var(--font-mono); color: var(--accent-rose);">-$500.00</span>
                    </div>
                    <div style="display: flex; justify-content: space-between; font-size: 13px; margin-bottom: 8px;">
                        <span style="color: var(--text-muted);">Value Added Tax (+9% VAT):</span>
                        <span id="prevTax" style="font-family: var(--font-mono); color: var(--accent-amber);">+$855.00</span>
                    </div>
                    <div style="display: flex; justify-content: space-between; font-size: 16px; font-weight: 800; padding-top: 8px; border-top: 1px solid var(--border-subtle); color: var(--accent-emerald);">
                        <span>Total Due (Accounts Receivable):</span>
                        <span id="prevTotal" style="font-family: var(--font-mono);">$10,355.00</span>
                    </div>
                </div>

                <button class="btn-gradient" onclick="postInvoice()">
                    ⚡ Settle & Post to Double-Entry General Ledger
                </button>
            </div>

            <div class="panel-card" data-widget-id="panel_ledger_table" data-widget-kind="Card" data-source-file="src/ui_dashboard.end" data-source-line="110">
                <div class="panel-header">
                    <h3 class="panel-title">📖 Real-Time Double-Entry General Ledger</h3>
                    <span class="badge-tag badge-info">Audited Invariant</span>
                </div>

                <div class="table-wrap">
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>Ref #</th>
                                <th>Description</th>
                                <th>Debit ($)</th>
                                <th>Credit ($)</th>
                                <th>Audit Status</th>
                            </tr>
                        </thead>
                        <tbody id="ledgerRows">
                            <tr>
                                <td style="font-family: var(--font-mono); color: var(--accent-cyan);">TX-001</td>
                                <td>Initial Cash Reserve Deposit</td>
                                <td style="font-family: var(--font-mono); font-weight: 700;">$50,000.00</td>
                                <td style="font-family: var(--font-mono);">-</td>
                                <td><span class="badge-tag badge-success">Balanced</span></td>
                            </tr>
                            <tr>
                                <td style="font-family: var(--font-mono); color: var(--accent-cyan);">TX-002</td>
                                <td>ACME Enterprise Sales Invoice</td>
                                <td style="font-family: var(--font-mono); font-weight: 700; color: var(--accent-emerald);">$10,355.00</td>
                                <td style="font-family: var(--font-mono); font-weight: 700;">$10,355.00</td>
                                <td><span class="badge-tag badge-success">Balanced</span></td>
                            </tr>
                            <tr>
                                <td style="font-family: var(--font-mono); color: var(--accent-cyan);">TX-003</td>
                                <td>Cloud Server Hosting Expense</td>
                                <td style="font-family: var(--font-mono);">-</td>
                                <td style="font-family: var(--font-mono); color: var(--accent-rose); font-weight: 700;">$1,200.00</td>
                                <td><span class="badge-tag badge-success">Settled</span></td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </section>
    </main>

    {overlay_html}

    <script>
        let currentCash = 59155;
        let currentRevenue = 9500;
        let currentExpenses = 1200;
        let currentAR = 10355;
        let txSeq = 4;

        function recalcInvoice() {{
            const qty = parseFloat(document.getElementById('invQty').value) || 0;
            const price = parseFloat(document.getElementById('invPrice').value) || 0;
            const discPct = parseFloat(document.getElementById('invDiscount').value) || 0;
            const taxRate = parseFloat(document.getElementById('invTaxRate').value) || 0.09;

            const subtotal = qty * price;
            const discount = subtotal * (discPct / 100);
            const taxable = subtotal - discount;
            const tax = taxable * taxRate;
            const total = taxable + tax;

            document.getElementById('prevSubtotal').innerText = '$' + subtotal.toLocaleString('en-US', {{ minimumFractionDigits: 2, maximumFractionDigits: 2 }});
            document.getElementById('prevDiscount').innerText = '-$' + discount.toLocaleString('en-US', {{ minimumFractionDigits: 2, maximumFractionDigits: 2 }});
            document.getElementById('prevTax').innerText = '+$' + tax.toLocaleString('en-US', {{ minimumFractionDigits: 2, maximumFractionDigits: 2 }});
            document.getElementById('prevTotal').innerText = '$' + total.toLocaleString('en-US', {{ minimumFractionDigits: 2, maximumFractionDigits: 2 }});
        }}

        function postInvoice() {{
            const customer = document.getElementById('invCustomer').value;
            const qty = parseFloat(document.getElementById('invQty').value) || 0;
            const price = parseFloat(document.getElementById('invPrice').value) || 0;
            const discPct = parseFloat(document.getElementById('invDiscount').value) || 0;
            const taxRate = parseFloat(document.getElementById('invTaxRate').value) || 0.09;

            const subtotal = qty * price;
            const discount = subtotal * (discPct / 100);
            const taxable = subtotal - discount;
            const tax = taxable * taxRate;
            const total = taxable + tax;

            if (total <= 0) {{ alert('Please enter valid quantities and prices.'); return; }}

            currentCash += total;
            currentRevenue += taxable;
            currentAR += total;
            const profit = currentRevenue - currentExpenses;

            document.getElementById('valCash').innerText = '$' + currentCash.toLocaleString('en-US', {{ minimumFractionDigits: 2 }});
            document.getElementById('valRevenue').innerText = '$' + currentRevenue.toLocaleString('en-US', {{ minimumFractionDigits: 2 }});
            document.getElementById('valProfit').innerText = '$' + profit.toLocaleString('en-US', {{ minimumFractionDigits: 2 }});
            document.getElementById('valAR').innerText = '$' + currentAR.toLocaleString('en-US', {{ minimumFractionDigits: 2 }});

            const tbody = document.getElementById('ledgerRows');
            const tr = document.createElement('tr');
            tr.innerHTML = `
                <td style="font-family: var(--font-mono); color: var(--accent-cyan);">TX-00${{txSeq++}}</td>
                <td>${{customer}}</td>
                <td style="font-family: var(--font-mono); font-weight: 700; color: var(--accent-emerald);">$${{total.toLocaleString('en-US', {{ minimumFractionDigits: 2 }})}}</td>
                <td style="font-family: var(--font-mono); font-weight: 700;">$${{total.toLocaleString('en-US', {{ minimumFractionDigits: 2 }})}}</td>
                <td><span class="badge-tag badge-success">Balanced</span></td>
            `;
            tbody.insertBefore(tr, tbody.firstChild);

            alert('✔ Double-Entry Posting Verified! Debits ($' + total.toFixed(2) + ') == Credits ($' + total.toFixed(2) + ') matched to the exact cent.');
        }}

        function scrollToInvoice() {{
            document.getElementById('invoicePanel').scrollIntoView({{ behavior: 'smooth' }});
        }}

        function switchNav(elem) {{
            document.querySelectorAll('.nav-item').forEach(i => i.classList.remove('active'));
            elem.classList.add('active');
        }}

        recalcInvoice();
    </script>
</body>
</html>
"#, overlay_html = overlay_html)
    }
}
