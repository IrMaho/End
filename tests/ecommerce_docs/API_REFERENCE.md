# 📖 main API & Technical Reference

> **Compiler Engine:** End Language `0.4.0-alpha (Enterprise Vibe Coding Edition)`  
> **Source Entrypoint:** `C:\Users\ASUS\Desktop\flutter_project\end\tests\test_ecommerce_service.end`  
> **Total Lines:** `110` lines  

## ⚡ HTTP REST Endpoints (OpenAPI 3.1 Compatible)

| Method | Path | Summary | Handler | Response Type |
| :--- | :--- | :--- | :--- | :--- |
| **GET** | `/api/v1/health` | Handler for get_health | `get_health` | `I32` |
| **GET** | `/api/v1/products` | Handler for list_products | `list_products` | `I64` |
| **POST** | `/api/v1/orders` | Handler for create_order | `create_order` | `Custom("OrderResponse")` |

## 📦 Struct Definitions & Memory Layout

### `st User`
*Core User profile entity*

- **Total Memory Size:** `32` Bytes
- **Hardware Alignment:** `8` Bytes

| Offset | Field | Type | Size | Alignment |
| :--- | :--- | :--- | :--- | :--- |
| `+0B` | `id` | `I64` | `8B` | `8B` |
| `+8B` | `username` | `Str` | `8B` | `8B` |
| `+16B` | `email` | `Str` | `8B` | `8B` |
| `+24B` | `is_vip` | `Bool` | `1B` | `1B` |

### `st Product`
*Product catalog item with pricing and stock*

- **Total Memory Size:** `32` Bytes
- **Hardware Alignment:** `8` Bytes

| Offset | Field | Type | Size | Alignment |
| :--- | :--- | :--- | :--- | :--- |
| `+0B` | `id` | `I64` | `8B` | `8B` |
| `+8B` | `name` | `Str` | `8B` | `8B` |
| `+16B` | `price_cents` | `I64` | `8B` | `8B` |
| `+24B` | `inventory_count` | `I64` | `8B` | `8B` |

### `st OrderItem`
*Individual item in an order line*

- **Total Memory Size:** `24` Bytes
- **Hardware Alignment:** `8` Bytes

| Offset | Field | Type | Size | Alignment |
| :--- | :--- | :--- | :--- | :--- |
| `+0B` | `product_id` | `I64` | `8B` | `8B` |
| `+8B` | `quantity` | `I64` | `8B` | `8B` |
| `+16B` | `unit_price_cents` | `I64` | `8B` | `8B` |

### `st CreateOrderRequest`
*Inbound payload for creating a new order*

- **Total Memory Size:** `24` Bytes
- **Hardware Alignment:** `8` Bytes

| Offset | Field | Type | Size | Alignment |
| :--- | :--- | :--- | :--- | :--- |
| `+0B` | `user_id` | `I64` | `8B` | `8B` |
| `+8B` | `items_count` | `I64` | `8B` | `8B` |
| `+16B` | `shipping_address` | `Str` | `8B` | `8B` |

### `st OrderResponse`
*Outbound response for order creation*

- **Total Memory Size:** `24` Bytes
- **Hardware Alignment:** `8` Bytes

| Offset | Field | Type | Size | Alignment |
| :--- | :--- | :--- | :--- | :--- |
| `+0B` | `order_id` | `I64` | `8B` | `8B` |
| `+8B` | `total_amount_cents` | `I64` | `8B` | `8B` |
| `+16B` | `status` | `Str` | `8B` | `8B` |

## ⚡ Functions & Invariants

### `fn get_health() -> I32`
- **Memory Safety Tier:** `Tier 1 (Arena Scoped / Zero-Alloc)`
- **Purity:** `Pure (Deterministic)`
- **Capabilities:** `pure, concurrency_safe`
- **Invariants:**
  - Idempotent: Identical inputs guarantee identical output with zero side-effects.

### `fn list_products() -> I64`
- **Memory Safety Tier:** `Tier 1 (Arena Scoped / Zero-Alloc)`
- **Purity:** `Pure (Deterministic)`
- **Capabilities:** `pure, concurrency_safe`
- **Invariants:**
  - Idempotent: Identical inputs guarantee identical output with zero side-effects.

### `fn create_order(req: Custom("CreateOrderRequest")) -> Custom("OrderResponse")`
- **Memory Safety Tier:** `Tier 1 (Arena Scoped / Zero-Alloc)`
- **Purity:** `Pure (Deterministic)`
- **Capabilities:** `pure, concurrency_safe`
- **Invariants:**
  - Idempotent: Identical inputs guarantee identical output with zero side-effects.

### `fn calculate_tax(amount_cents: I64, tax_rate_basis_points: I64) -> I64`
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

