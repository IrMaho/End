# Evolvable Modules, Multi-Dimensional Facets & Lifecycle

The End Programming Language redefines modularity with **Evolvable Modules** and **Multi-Dimensional Facets**.

---

## 🌟 1. Evolvable Modules (`@evolvable`)

Marking a module `@evolvable` directs the End compiler and Evolution Engine to continuously monitor its health, verify public ABI contracts, calculate extensibility metrics, and ensure clean separation of concerns.

```end
mod @evolvable UserStore {
    @facet(api) {
        pub fn find_by_id(id: i64) -> User;
        pub fn save(user: &User) -> Result<void, str>;
    }

    @facet(implementation) {
        fn execute_sql(q: str) {
            // internal implementation
        }
    }

    @facet(tests) {
        fn test_user_save() {
            assert!(true);
        }
    }

    @facet(extension) {
        extension_point on_user_saved(user: &User);
    }

    @facet(architecture) {
        owns: [UserRecord];
        depends_only: [SqlDriver];
    }
}
```

---

## 🔄 2. Module Replacement & Overlays

### Complete Replacement (`replace mod`)
```end
replace mod MockBilling with StripeBillingService {
    satisfies BillingContract;
}
```

### Module Overlay (`overlay mod`)
```end
overlay mod PaymentEngine for StagingEnvironment {
    override fn process_payment(amount: f64) -> bool {
        return true;
    }
}
```

---

## 📸 3. API Snapshots & SemVer Diff Engine

### Generating an API Snapshot:
```bash
end api snapshot src/user.end
```
Produces a canonical, cryptographically hashed JSON snapshot representing all public structs, fields, functions, and traits.

### Differential SemVer Analysis:
```bash
end api diff v1_snapshot.end --target-file v2_snapshot.end
```
Output:
- **`MAJOR`**: Detected removed public functions or modified parameter types.
- **`MINOR`**: Detected backward-compatible added public functions.
- **`PATCH`**: Purely internal implementation modifications.
