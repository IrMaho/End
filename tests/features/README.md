# End Language — Feature Test Suite for Language Consumers

> **Purpose:** Real-world .end programs demonstrating every major feature of the End language.  
> **Coverage:** All **100 syntactic & architectural features** across **16 thematic groups** — 2 files per group (32 files total).  
> **Audience:** End language users, library authors, and contributors who want idiomatic usage examples.

---

## 🗂 Index of Consumer Test Files

| File | Features Covered | Items |
|:-----|:----------------|:------|
| 01_destructuring_tuples_patterns.end | Multi-value return, Rest unpacking *, Wildcard _ | 1–3 |
| 01_named_destructuring_and_guards.end | Named struct destructuring, match guards, when | 4–6 |
| 02_list_dict_set_comprehensions.end | List / Dict / Set comprehensions | 7–9 |
| 02_conditional_and_walrus_expr.end | Ternary x if c else y, Walrus := | 10–11 |
| 03_variadic_args_kwargs.end | Variadic *args, keyword **kwargs | 12–13 |
| 03_named_optional_required_params.end | Named call, optional defaults, equired params | 14–16 |
| 04_null_aware_access_coalescing.end | ?. access, ?? coalescing, ??= assignment | 17–20 |
| 04_dart_cascades_and_null_cascades.end | Cascade .., null-aware cascade ?.. | 21–22 |
| 05_collection_spreads_and_null_spreads.end | Spread ..., null-aware spread ...? | 23–24 |
| 05_collection_if_and_for_flow.end | Collection if, collection or, nested control flow | 25–27 |
| 06_extension_methods_and_properties.end | extend Type { fn ... }, extension properties | 28–29 |
| 06_operator_overloading_and_invoke.end | Operator overloading, invoke convention | 30–31 |
| 07_destructuring_protocol_and_copy.end | Destructuring protocol, .copy(...) | 32–33 |
| 07_delegated_properties_and_wrappers.end | y / using delegation, property wrappers | 34–35 |
| 08_result_builders_and_dsl.end | Result builders, declarative DSL blocks | 36 |
| 08_trailing_closures_and_implicit_lambdas.end | Trailing closures, implicit _ lambda | 37–38 |
| 09_pipe_operator_and_ranges.end | Pipe \|>, ranges .. / ..<, spread in call | 39–41 |
| 09_string_interpolation_and_raw_strings.end | String interpolation ${}, raw strings | 42–43 |
| 10_enum_payloads_and_expression_match.end | Enums with payloads, expression if/match/blocks | 44–47 |
| 10_local_functions_and_trait_defaults.end | Local functions, default trait implementations, pattern binding | 48–50 |
| 11_composition_use_equip_attach.end | use, equip ... with, ttach ... to, detach | 51–54 |
| 11_shapes_surfaces_views_fuse.end | shape, iew, compose, use, ugment, surface | 55–60 |
| 12_require_resolve_context_scopes.end | equire, esolve, context, scope | 61–65 |
| 12_grant_deny_expose_seal_gates.end | grant, deny, expose, seal | 66–69 |
| 13_hooks_interceptors_decorators.end | hook, intercept, decorate | 70–72 |
| 13_replace_delegate_guards.end | eplace, delegate, guard, when reactive | 73–76 |
| 14_borrow_leases_and_when_conditions.end | orrow, only ... can, when constraint | 77–78 |
| 14_formal_proofs_assume_guarantee.end | ssume, guarantee, prove, expect, ecause, eplaceable, evolvable | 81–87 |
| 15_first_class_features_variants_fallbacks.end | eature, extends, uses, optional, allback, ariants | 88–93 |
| 15_architectural_policies_and_contracts.end | policy, rchitecture, contract, implement | 94–97 |
| 16_agent_skills_tasks_and_master_evolution.end | skill, 	ask, evolve, impact | 79–80, 98–99 |
| 16_autonomous_verified_full_pipeline.end | Full evolve + feature + skill + verify pipeline | 100 |

---

## Running the Tests

`
# Check syntax (parse only)
endc check tests/features/01_destructuring_tuples_patterns.end

# Run a specific test
endc run tests/features/01_destructuring_tuples_patterns.end

# Batch-check all feature tests (PowerShell)
Get-ChildItem tests/features/*.end | ForEach-Object { endc check .FullName }
`

---

## Design Principles

Each test file:
1. Covers real language features — not toy snippets, but idiomatic End code patterns
2. Is self-contained — no external dependencies beyond the End stdlib
3. Has a main() entry point returning i64 for executable verification
4. Is <= 100 lines — readable at a glance by any language consumer
5. Includes comments mapping to feature item numbers for cross-referencing with END_100_SYNTAX_SPECIFICATION.md

---

End Language Feature Consumer Test Suite - v1.0.0 - 32 files - 100 features - 100% documented
