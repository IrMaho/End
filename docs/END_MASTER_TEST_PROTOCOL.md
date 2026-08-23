# 🎯 End Language — Master Development & Unit Test Protocol (v3.0)

## 📐 پرامپت تخصصی برای توسعه و تست زبان End

---

## 🌐 Context کامل پروژه

**زبان End** یک زبان برنامه‌نویسی انقلابی با کامپایلر Rust است که در:

```
c:\Users\ASUS\Desktop\flutter_project\end\
├── endc\              # کامپایلر Rust (endc.exe)
│   └── src\
│       ├── lexer\     # tokens.rs, keywords.rs, operator.rs
│       ├── parser\    # decl/, stmt/, expr/
│       ├── ast\       # decl/, expr/, stmt/, pattern/
│       ├── ir\        # MIR pipeline
│       └── cli\       # commands.rs
├── tests\
│   ├── features\      # 32 consumer .end tests (100% PASSING)
│   └── *.end          # system integration tests
└── docs\              # 55 specification documents
```

**کامپایلر:** `endc\target\release\endc.exe`
**Branch فعال:** `feat/majul`

---

## 🏗️ معماری و قوانین الزامی

### قانون ۱: محدودیت خطوط
```
هر فایل .rs یا .end باید STRICTLY ≤ 500 خط باشد.
```

### قانون ۲: ساختار ماژولار (SOLID)
- هر ماژول یک مسئولیت واحد دارد
- Parser = تجزیه → Semantic = بررسی → IR = تبدیل → Codegen = تولید کد
- هیچ import browser/window/DOM ای در کامپایلر مجاز نیست

### قانون ۳: سینتکس واقعی Parser (Verified ✅)

```end
// ─── Null Safety ───
user?.profile?.name          // null-aware access
name ?? "Guest"              // null coalescing
cache ??= create_cache()     // null-coalescing assign

// ─── Destructuring ───
(id, name, email) := user()  // multi-value unpack
(head, *rest) := values      // rest unpacking
User{id, email} := user      // named struct destructuring

// ─── Comprehensions ───
[x * 2 for x in xs if x > 0]          // list comprehension
{k: v for k, v in map if v.active}    // dict comprehension
{x.id for x in users}                 // set comprehension

// ─── Capabilities ───
use DatabaseConnection;
equip User with { AdminCapabilities }
attach { Logging } to Service;
detach Logging from OldService;

// ─── Resolve (ARROW نه :=) ───
resolve PaymentProvider -> StripeProvider;
resolve Storage -> RedisCluster when production;

// ─── Grant/Deny ───
grant Payments.refund { FinanceAdmin }
deny Payments { database }

// ─── Shape/Surface/View ───
surface Payment.public { pay; refund; }
shape User.Public { id; name; email; }
view User as PublicUser;
fuse { Search, Pagination } as SearchableList;
augment User { capability Searchable; }

// ─── Hook/Intercept/Decorate ───
hook Payment.after_refund { AuditLogger.log(); }
intercept Payment.refund { before { check(); } after { log(); } }
decorate Service with { Metrics, Logging, }

// ─── Delegate/Borrow ───
delegate BillingInterface.charge to StripeService;
borrow HardwareGpuBuffer;        // NO 'for' keyword!

// ─── Guard (داخل fn body) ───
fn process(role: string) -> i64 {
    guard role == "admin" else { ret 1; }
    ret 0;
}

// ─── Architecture ───
architecture CleanArch {
    UI -> Application          // NO semicolons!
    Application -> Domain
}

// ─── Policy (داخل feature) ───
feature SecurePayment {
    replaceable;
    policy { no_plaintext_passwords; enforce_tls; }
}

// ─── Contract/Implement ───
contract PaymentContract {
    process_charge(cents: i64) -> bool
    refund_charge(id: string) -> bool
}
implement PaymentContract for StripeService {
    fn process_charge(cents: i64) -> bool { ret true; }
}

// ─── Extension ───
extend string { fn is_email() -> bool { ret true; } }

// ─── Pipes & Ranges ───
data |> parse |> validate |> save
let r = 0..<10;

// ─── String Interpolation ───
val msg = "Hello $name, count: ${items.len()}";

// ─── Pattern Matching ───
match result {
    Ok(v) => v.process(),
    Error(code) => handle(code),
}

// ─── Enum with Payloads ───
enum NetworkState { Idle, Loading(i64), Success(string) }

// ─── Skill/Task/Evolve ───
skill AutoRefactor { require ASTInspection; }
task MigrateToV2 { target UserDatabase; requires AutoRefactor; }
evolve DatabaseSchema { add UserEmailVerified; }
```

---

## ✅ وضعیت فعلی پروژه

| دسته | وضعیت |
|:-----|:-------|
| 100 فیچر سینتکسی | ✅ پیاده‌سازی کامل (Commit: c0f0434) |
| 300 یونیت تست داخلی | ✅ 100% پاس |
| 32 consumer test به زبان End | ✅ 32/32 PASSING (Commit: aa46e34) |
| 55 سند مستندات | ✅ در docs/ |
| معماری ماژولار | ✅ همه فایل‌ها ≤ 500 خط |

---

## 📋 پروتکل اجرایی برای هر فیچر جدید

### مرحله ۱: پلن‌نویسی
```
- سینتکس دقیق: چه توکن‌هایی parse می‌شن؟
- AST node: چه struct/enum ای در ast/ ایجاد می‌شه؟
- IR lowering: چطور به MIR تبدیل می‌شه؟
- Codegen: چه کدی تولید می‌کنه؟
```

### مرحله ۲: پیاده‌سازی (به ترتیب)
```
1. lexer/keywords.rs  → TokenKind اضافه کن
2. ast/              → AST node تعریف کن
3. parser/decl/ یا parser/stmt/ → parse fn بنویس
4. ir/              → lowering پیاده‌سازی کن
5. codegen/         → code generation اضافه کن
```

### مرحله ۳: تست‌نویسی (الزامی!)
هر فیچر **حداقل ۲ تست به زبان End** + **۳ یونیت‌تست Rust** باید داشته باشه:

#### الگوی تست consumer (tests/features/XX_name.end):
```end
// ============================================================================
// End Language Consumer Test: <Feature Name>
// Covers:
//   - Item NN: `keyword <syntax>` — توضیح
// ============================================================================

// 1. Test Case: <scenario description>
<feature syntax here>

// 2. Test Case: <complex composed scenario>
<composed usage>

fn main() -> i64 {
    // assertions
    ret 0;
}
```

#### الگوی یونیت‌تست Rust (endc/src/tests/ یا endc/src/ir/):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::lexer::Lexer;

    fn parse_end(src: &str) -> Result<Vec<Statement>, String> {
        let tokens = Lexer::new(src).tokenize();
        Parser::new(tokens).parse()
    }

    #[test]
    fn test_feature_basic() {
        // Test 1: Basic positive case
        let result = parse_end("resolve X -> Provider;");
        assert!(result.is_ok(), "Expected OK, got: {:?}", result);
    }

    #[test]
    fn test_feature_composed() {
        // Test 2: Complex composition
        let result = parse_end(r#"
            feature Payments {
                require Database;
                resolve DB -> PostgresPool;
            }
        "#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_feature_edge_case() {
        // Test 3: Edge/Error case
        let result = parse_end("resolve ;"); // missing target
        assert!(result.is_err());
    }
}
```

### مرحله ۴: اجرا و تأیید
```powershell
# Build
cd endc; cargo build --release

# Check consumer test
.\target\release\endc.exe check ..\tests\features\XX_feature.end

# Run all consumer tests (باید 32/32 پاس بشه)
$endc = ".\endc\target\release\endc.exe"
Get-ChildItem tests\features\*.end | Sort-Object Name | ForEach-Object {
    $r = & $endc check $_.FullName 2>&1
    if ($LASTEXITCODE -eq 0) { "✅ PASS $_Name" } else { "❌ FAIL $($_.Name): $r" }
}

# Run Rust unit tests
cd endc; cargo test 2>&1 | tail -5
```

### مرحله ۵: مستندسازی
```
1. docs/ → فایل مستنداتی به نام FEATURE_NAME.md ایجاد یا بروزرسانی کن
2. آرتیفکت roadmap را بروزرسانی کن (علامت ✅ بزن)
3. git add + git commit با پیام استاندارد:
   feat(parser): add <feature> with <N> tests
```

---

## 🚫 خطاهای رایج — هرگز نکن!

```end
// ❌ WRONG: resolve با :=
resolve X := Provider;

// ✅ CORRECT: resolve با ->
resolve X -> Provider;

// ❌ WRONG: grant با to
grant Refund to FinanceAdmin;

// ✅ CORRECT: grant با brace-list
grant Payments.refund { FinanceAdmin }

// ❌ WRONG: borrow با for
borrow GPU for session;

// ✅ CORRECT: borrow فقط path
borrow GPU;

// ❌ WRONG: intercept با with
intercept Payment.refund with Handler;

// ✅ CORRECT: intercept با block
intercept Payment.refund { before { check(); } after { log(); } }

// ❌ WRONG: shape با = و surface
shape User.Public = User surface { id; }

// ✅ CORRECT: shape با brace
shape User.Public { id; name; }

// ❌ WRONG: policy به عنوان top-level statement
policy DataPolicy { no_plain; }

// ✅ CORRECT: policy داخل feature
feature SecureSystem { policy { no_plain; } }

// ❌ WRONG: architecture با semicolon
architecture Clean { UI -> App; App -> Domain; }

// ✅ CORRECT: architecture بدون semicolon
architecture Clean { UI -> App \n App -> Domain }

// ❌ WRONG: guard به عنوان top-level
guard condition { enable X for Y; } else { fallback Z; }

// ✅ CORRECT: guard داخل fn
fn process() -> i64 { guard condition else { ret 1; } ret 0; }

// ❌ WRONG: UTF-8 BOM در فایل‌های .end
// (PowerShell Set-Content -Encoding UTF8 اضافه می‌کنه)

// ✅ CORRECT: بدون BOM
[System.IO.File]::WriteAllText(path, content, [System.Text.UTF8Encoding]::new($false))
```

---

## 📊 ساختار درختی برای فیچرهای جدید

هر فیچر باید این ۵ لایه رو cover کنه:

```
Feature X
├── 📄 Spec       → docs/FEATURE_X.md
├── 🔤 Token      → lexer/keywords.rs (TokenKind::X)
├── 🌳 AST        → ast/decl/feature_x.rs یا ast/stmt/feature_x.rs
├── 🔍 Parser     → parser/decl/feature_x.rs یا parser/stmt/feature_x.rs
├── ⚙️ IR         → ir/feature_x_lower.rs
├── 🧪 Tests
│   ├── tests/features/XX_feature_x.end       (consumer test 1)
│   ├── tests/features/XX_feature_x_advanced.end (consumer test 2)
│   └── endc/src/tests/feature_x_test.rs      (3 unit tests)
└── 📝 Commit     → feat(parser): add X with N tests
```

---

## 🏆 استاندارد کیفیت — ملاک قبولی

| معیار | استاندارد |
|:------|:----------|
| consumer tests | ✅ 32/32 PASSING (حفظ شود) |
| unit tests | ≥ 3 test per feature |
| file size | ≤ 500 lines per file |
| parser coverage | همه سینتکس‌های جدید از طریق `endc check` تأیید شده |
| docs | هر فیچر مستند دارد |
| git | commit پیام استاندارد: `feat/fix/docs(scope): message` |

---

## 📚 فایل‌های کلیدی مرجع

```
Parser Rules:     endc/src/parser/stmt/capabilities.rs
                  endc/src/parser/stmt/capability_extensions.rs
                  endc/src/parser/decl/contracts.rs
Token Defs:       endc/src/lexer/keywords.rs
AST Types:        endc/src/ast/statement.rs
Consumer Tests:   tests/features/ (32 verified files)
Docs:             docs/END_100_SYNTAX_SPECIFICATION.md
Roadmap:          docs/ROADMAP.md
```

---

## 💡 پرامپت آماده برای فیچر جدید (Copy-Paste Ready)

```
عشقم، می‌خوام فیچر جدید "<FEATURE_NAME>" رو به زبان End اضافه کنیم.

**مشخصات:**
- سینتکس: `<exact syntax here>`
- کاربرد: <use case description>
- مثال واقعی: <complex real-world example>

**لازم الاجرا:**
1. پلن کامل پیاده‌سازی (Lexer → AST → Parser → IR → Codegen)
2. حداقل ۲ consumer test به زبان End در tests/features/
3. حداقل ۳ unit test Rust در endc/src/
4. مستندات در docs/FEATURE_NAME.md
5. اجرا با `endc check` و تأیید 32/32 consumer tests حفظ بشه
6. آرتیفکت roadmap بروزرسانی بشه
7. commit با پیام استاندارد روی branch feat/majul

**هر فایل جدید باید ≤ 500 خط باشد.**
**فایل‌های .end باید بدون BOM نوشته بشن (UTF8NoBOM).**
```

---

*End Language Master Protocol v3.0 — Verified against commit aa46e34 on feat/majul*
*32/32 Consumer Tests PASSING | 536/536 Unit Tests PASSING | 100 Features COMPLETE*