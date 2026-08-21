# 📖 main API & Technical Reference

> **Compiler Engine:** End Language `0.4.0-alpha (Enterprise Vibe Coding Edition)`  
> **Source Entrypoint:** `C:\Users\ASUS\Desktop\flutter_project\end\examples\end_ledger\src\main.end`  
> **Total Lines:** `304` lines  

## ⚡ HTTP REST Endpoints (OpenAPI 3.1 Compatible)

| Method | Path | Summary | Handler | Response Type |
| :--- | :--- | :--- | :--- | :--- |
| **GET** | `/api/v1/accounts` | Handler for get_accounts | `get_accounts` | `I64` |
| **POST** | `/api/v1/ledger/post` | Handler for post_journal_entry | `post_journal_entry` | `Bool` |
| **GET** | `/api/v1/reports/financial-summary` | Handler for get_financial_summary | `get_financial_summary` | `Custom("FinancialReport")` |

## 📦 Struct Definitions & Memory Layout

### `st Account`
*Core financial account entity*

- **Total Memory Size:** `40` Bytes
- **Hardware Alignment:** `8` Bytes

| Offset | Field | Type | Size | Alignment |
| :--- | :--- | :--- | :--- | :--- |
| `+0B` | `id` | `I64` | `8B` | `8B` |
| `+8B` | `code` | `Str` | `8B` | `8B` |
| `+16B` | `name` | `Str` | `8B` | `8B` |
| `+24B` | `balance_cents` | `I64` | `8B` | `8B` |
| `+32B` | `account_type_id` | `I64` | `8B` | `8B` |

### `st JournalEntry`
*Ledger transaction journal header*

- **Total Memory Size:** `48` Bytes
- **Hardware Alignment:** `8` Bytes

| Offset | Field | Type | Size | Alignment |
| :--- | :--- | :--- | :--- | :--- |
| `+0B` | `id` | `I64` | `8B` | `8B` |
| `+8B` | `reference_no` | `Str` | `8B` | `8B` |
| `+16B` | `description` | `Str` | `8B` | `8B` |
| `+24B` | `total_debit_cents` | `I64` | `8B` | `8B` |
| `+32B` | `total_credit_cents` | `I64` | `8B` | `8B` |
| `+40B` | `is_posted` | `Bool` | `1B` | `1B` |

### `st LedgerLine`
*Individual debit/credit line in a journal entry*

- **Total Memory Size:** `40` Bytes
- **Hardware Alignment:** `8` Bytes

| Offset | Field | Type | Size | Alignment |
| :--- | :--- | :--- | :--- | :--- |
| `+0B` | `id` | `I64` | `8B` | `8B` |
| `+8B` | `entry_id` | `I64` | `8B` | `8B` |
| `+16B` | `account_id` | `I64` | `8B` | `8B` |
| `+24B` | `debit_cents` | `I64` | `8B` | `8B` |
| `+32B` | `credit_cents` | `I64` | `8B` | `8B` |

### `st Customer`
*Customer profile with credit limit*

- **Total Memory Size:** `40` Bytes
- **Hardware Alignment:** `8` Bytes

| Offset | Field | Type | Size | Alignment |
| :--- | :--- | :--- | :--- | :--- |
| `+0B` | `id` | `I64` | `8B` | `8B` |
| `+8B` | `name` | `Str` | `8B` | `8B` |
| `+16B` | `tax_id` | `Str` | `8B` | `8B` |
| `+24B` | `credit_limit_cents` | `I64` | `8B` | `8B` |
| `+32B` | `outstanding_balance_cents` | `I64` | `8B` | `8B` |

### `st InvoiceItem`
*Itemized invoice line*

- **Total Memory Size:** `40` Bytes
- **Hardware Alignment:** `8` Bytes

| Offset | Field | Type | Size | Alignment |
| :--- | :--- | :--- | :--- | :--- |
| `+0B` | `id` | `I64` | `8B` | `8B` |
| `+8B` | `product_name` | `Str` | `8B` | `8B` |
| `+16B` | `quantity` | `I64` | `8B` | `8B` |
| `+24B` | `unit_price_cents` | `I64` | `8B` | `8B` |
| `+32B` | `total_cents` | `I64` | `8B` | `8B` |

### `st Invoice`
*Customer sales invoice*

- **Total Memory Size:** `64` Bytes
- **Hardware Alignment:** `8` Bytes

| Offset | Field | Type | Size | Alignment |
| :--- | :--- | :--- | :--- | :--- |
| `+0B` | `id` | `I64` | `8B` | `8B` |
| `+8B` | `customer_id` | `I64` | `8B` | `8B` |
| `+16B` | `invoice_number` | `Str` | `8B` | `8B` |
| `+24B` | `subtotal_cents` | `I64` | `8B` | `8B` |
| `+32B` | `tax_cents` | `I64` | `8B` | `8B` |
| `+40B` | `discount_cents` | `I64` | `8B` | `8B` |
| `+48B` | `total_cents` | `I64` | `8B` | `8B` |
| `+56B` | `is_paid` | `Bool` | `1B` | `1B` |

### `st FinancialReport`
*Comprehensive Financial Health & Income Statement Report*

- **Total Memory Size:** `40` Bytes
- **Hardware Alignment:** `8` Bytes

| Offset | Field | Type | Size | Alignment |
| :--- | :--- | :--- | :--- | :--- |
| `+0B` | `total_revenue_cents` | `I64` | `8B` | `8B` |
| `+8B` | `total_expenses_cents` | `I64` | `8B` | `8B` |
| `+16B` | `net_income_cents` | `I64` | `8B` | `8B` |
| `+24B` | `total_assets_cents` | `I64` | `8B` | `8B` |
| `+32B` | `total_liabilities_cents` | `I64` | `8B` | `8B` |

### `st Card`
- **Total Memory Size:** `24` Bytes
- **Hardware Alignment:** `8` Bytes

| Offset | Field | Type | Size | Alignment |
| :--- | :--- | :--- | :--- | :--- |
| `+0B` | `title` | `Str` | `8B` | `8B` |
| `+8B` | `subtitle` | `Str` | `8B` | `8B` |
| `+16B` | `button_action` | `Str` | `8B` | `8B` |

### `st Text`
- **Total Memory Size:** `32` Bytes
- **Hardware Alignment:** `8` Bytes

| Offset | Field | Type | Size | Alignment |
| :--- | :--- | :--- | :--- | :--- |
| `+0B` | `text` | `Str` | `8B` | `8B` |
| `+8B` | `color` | `Str` | `8B` | `8B` |
| `+16B` | `font_size` | `Str` | `8B` | `8B` |
| `+24B` | `font_weight` | `Str` | `8B` | `8B` |

## ⚡ Functions & Invariants

### `fn get_accounts() -> I64`
- **Memory Safety Tier:** `Tier 1 (Arena Scoped / Zero-Alloc)`
- **Purity:** `Pure (Deterministic)`
- **Capabilities:** `pure, concurrency_safe`
- **Invariants:**
  - Idempotent: Identical inputs guarantee identical output with zero side-effects.

### `fn post_journal_entry(debit: I64, credit: I64) -> Bool`
- **Memory Safety Tier:** `Tier 1 (Arena Scoped / Zero-Alloc)`
- **Purity:** `Pure (Deterministic)`
- **Capabilities:** `pure, concurrency_safe`
- **Invariants:**
  - Idempotent: Identical inputs guarantee identical output with zero side-effects.

### `fn get_financial_summary() -> Custom("FinancialReport")`
- **Memory Safety Tier:** `Tier 1 (Arena Scoped / Zero-Alloc)`
- **Purity:** `Pure (Deterministic)`
- **Capabilities:** `pure, concurrency_safe`
- **Invariants:**
  - Idempotent: Identical inputs guarantee identical output with zero side-effects.

### `fn App() -> Custom("Card")`
- **Memory Safety Tier:** `Tier 1 (Arena Scoped / Zero-Alloc)`
- **Purity:** `Pure (Deterministic)`
- **Capabilities:** `pure, concurrency_safe`
- **Invariants:**
  - Idempotent: Identical inputs guarantee identical output with zero side-effects.

### `fn run_accounting_simulation() -> I32`
- **Memory Safety Tier:** `Tier 1 (Arena Scoped / Zero-Alloc)`
- **Purity:** `Pure (Deterministic)`
- **Capabilities:** `pure, concurrency_safe`
- **Invariants:**
  - Idempotent: Identical inputs guarantee identical output with zero side-effects.

### `fn main() -> I32`
- **Memory Safety Tier:** `Tier 1 (Arena Scoped / Zero-Alloc)`
- **Purity:** `Pure (Deterministic)`
- **Capabilities:** `pure, concurrency_safe`
- **Invariants:**
  - Idempotent: Identical inputs guarantee identical output with zero side-effects.

