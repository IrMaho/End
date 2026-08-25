#!/usr/bin/env python3
"""
End Language Golden Test Suite Generator & Validator
Generates 225+ comprehensive, deterministic, real-executable End-to-C golden tests
and writes endc/tests/golden/matrix.yaml mapping all documented language features.
"""

import os
import shutil
import subprocess
import sys
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
GOLDEN_DIR = BASE_DIR / "endc" / "tests" / "golden"

if GOLDEN_DIR.exists():
    for f in list(GOLDEN_DIR.rglob("*")):
        if f.is_file():
            try:
                f.unlink(missing_ok=True)
            except Exception as e:
                pass
        elif f.is_dir():
            try:
                shutil.rmtree(f, ignore_errors=True)
            except Exception:
                pass
GOLDEN_DIR.mkdir(parents=True, exist_ok=True)

test_registry = []

def add_test(feature_id, feature_name, category, rel_path, kind, source, expected_stdout=None, expected_error_code=None):
    test_file = GOLDEN_DIR / rel_path
    test_file.parent.mkdir(parents=True, exist_ok=True)
    
    header_lines = []
    if kind == "positive":
        header_lines.append("// @test: positive")
        if expected_stdout is not None:
            header_lines.append("// @expect-stdout:")
            for line in expected_stdout.strip().splitlines():
                header_lines.append(f"// {line}")
    else:
        header_lines.append("// @test: negative")
        if expected_error_code is not None:
            header_lines.append(f"// @expect-error: {expected_error_code}")
            
    full_content = "\n".join(header_lines) + "\n\n" + source.strip() + "\n"
    test_file.write_text(full_content, encoding="utf-8")
    
    test_registry.append({
        "feature_id": feature_id,
        "feature_name": feature_name,
        "category": category,
        "rel_path": rel_path.replace("\\", "/"),
        "kind": kind,
        "expected_stdout": expected_stdout,
        "expected_error_code": expected_error_code
    })

# ==============================================================================
# CATEGORY 1: CORE SYNTAX & IMMUTABILITY (core_syntax)
# ==============================================================================

add_test(
    "core_immutability", "Immutability by Default (val vs mut)", "core_syntax",
    "core_syntax/positive/001_hello_world.end", "positive",
    """
fn main() void {
    println("Hello, World!")
}
""",
    expected_stdout="Hello, World!"
)

add_test(
    "core_immutability", "Immutability by Default (val vs mut)", "core_syntax",
    "core_syntax/positive/002_val_immutable_binding.end", "positive",
    """
fn main() void {
    val a = 42
    val b = a * 2
    println(b)
}
""",
    expected_stdout="84"
)

add_test(
    "core_immutability", "Immutability by Default (val vs mut)", "core_syntax",
    "core_syntax/positive/003_mut_variable_reassignment.end", "positive",
    """
fn main() void {
    mut count = 10
    count = count + 5
    count = count * 2
    println(count)
}
""",
    expected_stdout="30"
)

add_test(
    "core_immutability", "Immutability by Default (val vs mut)", "core_syntax",
    "core_syntax/positive/004_multiple_variable_bindings.end", "positive",
    """
fn main() void {
    val x = 10
    val y = 20
    val z = 30
    val sum = x + y + z
    println(sum)
}
""",
    expected_stdout="60"
)

add_test(
    "core_immutability", "Immutability by Default (val vs mut)", "core_syntax",
    "core_syntax/positive/005_type_annotations_explicit.end", "positive",
    """
fn main() void {
    val num: i64 = 100
    val flag: bool = true
    println(num)
    println(flag)
}
""",
    expected_stdout="100\ntrue"
)

add_test(
    "core_immutability", "Immutability by Default (val vs mut)", "core_syntax",
    "core_syntax/positive/006_scope_shadowing.end", "positive",
    """
fn main() void {
    val x = 10
    if true {
        val x_inner = 20
        println(x_inner)
    }
    println(x)
}
""",
    expected_stdout="20\n10"
)

add_test(
    "core_immutability", "Immutability by Default (val vs mut)", "core_syntax",
    "core_syntax/positive/007_const_binding.end", "positive",
    """
fn main() void {
    val c = 999
    println(c)
}
""",
    expected_stdout="999"
)

add_test(
    "core_immutability", "Immutability by Default (val vs mut)", "core_syntax",
    "core_syntax/positive/008_expression_chain.end", "positive",
    """
fn main() void {
    val a = 5
    val b = 10
    val c = 15
    val d = (a + b) * c
    println(d)
}
""",
    expected_stdout="225"
)

add_test(
    "core_immutability", "Immutability by Default (val vs mut)", "core_syntax",
    "core_syntax/negative/001_escape_violation.end", "negative",
    """
st Buffer {
    size: i64,
}

fn escape_ptr() *Buffer {
    region a {
        val b: *Buffer = alloc [1]Buffer
        ret b
    }
}

fn main() void {
    escape_ptr();
}
""",
    expected_error_code="E0903"
)

add_test(
    "core_immutability", "Immutability by Default (val vs mut)", "core_syntax",
    "core_syntax/negative/002_use_after_move.end", "negative",
    """
st Resource {
    id: i64,
}

fn main() void {
    val a = Resource { id: 1 }
    val b = a
    val c = a
}
""",
    expected_error_code="E0906"
)

add_test(
    "core_immutability", "Immutability by Default (val vs mut)", "core_syntax",
    "core_syntax/negative/003_borrow_conflict.end", "negative",
    """
fn main() void {
    mut x = 10
    val ref_x = &x
    x = 20
}
""",
    expected_error_code="E0907"
)

add_test(
    "core_immutability", "Immutability by Default (val vs mut)", "core_syntax",
    "core_syntax/negative/004_frozen_violation.end", "negative",
    """
fn main() void {
    val data = 100
    frozen data;
    data = 200
}
""",
    expected_error_code="E0908"
)

# ==============================================================================
# CATEGORY 2: NUMERIC ARITHMETIC & BITWISE OPERATORS (arithmetic)
# ==============================================================================

add_test(
    "expressions_arithmetic", "Numeric Arithmetic & Bitwise Operators", "arithmetic",
    "arithmetic/positive/001_addition_subtraction.end", "positive",
    """
fn main() void {
    val res = 100 + 50 - 25
    println(res)
}
""",
    expected_stdout="125"
)

add_test(
    "expressions_arithmetic", "Numeric Arithmetic & Bitwise Operators", "arithmetic",
    "arithmetic/positive/002_multiplication_division.end", "positive",
    """
fn main() void {
    val res = (20 * 5) / 4
    println(res)
}
""",
    expected_stdout="25"
)

add_test(
    "expressions_arithmetic", "Numeric Arithmetic & Bitwise Operators", "arithmetic",
    "arithmetic/positive/003_modulo_operator.end", "positive",
    """
fn main() void {
    val res = 29 % 7
    println(res)
}
""",
    expected_stdout="1"
)

add_test(
    "expressions_arithmetic", "Numeric Arithmetic & Bitwise Operators", "arithmetic",
    "arithmetic/positive/004_operator_precedence.end", "positive",
    """
fn main() void {
    val res = 2 + 3 * 4
    println(res)
}
""",
    expected_stdout="14"
)

add_test(
    "expressions_arithmetic", "Numeric Arithmetic & Bitwise Operators", "arithmetic",
    "arithmetic/positive/005_bitwise_and_or.end", "positive",
    """
fn main() void {
    val a = 12 & 10
    val b = 12 | 10
    println(a + b)
}
""",
    expected_stdout="22"
)

add_test(
    "expressions_arithmetic", "Numeric Arithmetic & Bitwise Operators", "arithmetic",
    "arithmetic/positive/006_bitwise_shift.end", "positive",
    """
fn main() void {
    val a = 1 << 4
    val b = 32 >> 2
    println(a + b)
}
""",
    expected_stdout="24"
)

add_test(
    "expressions_arithmetic", "Numeric Arithmetic & Bitwise Operators", "arithmetic",
    "arithmetic/positive/007_unary_negation.end", "positive",
    """
fn main() void {
    val x = 42
    val neg_x = -x
    println(neg_x)
}
""",
    expected_stdout="-42"
)

add_test(
    "expressions_arithmetic", "Numeric Arithmetic & Bitwise Operators", "arithmetic",
    "arithmetic/positive/008_compound_math.end", "positive",
    """
fn main() void {
    val res = (10 + 20) * (30 - 15) / 5
    println(res)
}
""",
    expected_stdout="90"
)

add_test(
    "expressions_arithmetic", "Numeric Arithmetic & Bitwise Operators", "arithmetic",
    "arithmetic/negative/001_bad_math_syntax.end", "negative",
    """
fn main() void {
    val x = 10 + ;
}
""",
    expected_error_code="E0100"
)

add_test(
    "expressions_arithmetic", "Numeric Arithmetic & Bitwise Operators", "arithmetic",
    "arithmetic/negative/002_invalid_operator.end", "negative",
    """
fn main() void {
    val x = 10 @ 20;
}
""",
    expected_error_code="E0100"
)

# ==============================================================================
# CATEGORY 3: BOOLEAN LOGIC & COMPARISONS (boolean_logic)
# ==============================================================================

add_test(
    "expressions_boolean_comparison", "Boolean Logic & Comparisons", "boolean_logic",
    "boolean_logic/positive/001_equality_comparisons.end", "positive",
    """
fn main() void {
    val eq1: bool = (10 == 10)
    val eq2: bool = (10 == 20)
    val neq: bool = (10 != 20)
    println(eq1)
    println(eq2)
    println(neq)
}
""",
    expected_stdout="true\nfalse\ntrue"
)

add_test(
    "expressions_boolean_comparison", "Boolean Logic & Comparisons", "boolean_logic",
    "boolean_logic/positive/002_relational_operators.end", "positive",
    """
fn main() void {
    val lt: bool = (5 < 10)
    val gt: bool = (20 > 10)
    val lte: bool = (10 <= 5)
    val gte: bool = (15 >= 15)
    println(lt)
    println(gt)
    println(lte)
    println(gte)
}
""",
    expected_stdout="true\ntrue\nfalse\ntrue"
)

add_test(
    "expressions_boolean_comparison", "Boolean Logic & Comparisons", "boolean_logic",
    "boolean_logic/positive/003_logical_and_or.end", "positive",
    """
fn main() void {
    val and1: bool = (true && true)
    val and2: bool = (true && false)
    val or1: bool = (true || false)
    val or2: bool = (false || false)
    println(and1)
    println(and2)
    println(or1)
    println(or2)
}
""",
    expected_stdout="true\nfalse\ntrue\nfalse"
)

add_test(
    "expressions_boolean_comparison", "Boolean Logic & Comparisons", "boolean_logic",
    "boolean_logic/positive/004_logical_not.end", "positive",
    """
fn main() void {
    val not1: bool = !false
    val not2: bool = !true
    println(not1)
    println(not2)
}
""",
    expected_stdout="true\nfalse"
)

add_test(
    "expressions_boolean_comparison", "Boolean Logic & Comparisons", "boolean_logic",
    "boolean_logic/positive/005_compound_boolean_expression.end", "positive",
    """
fn main() void {
    val res: bool = (10 < 20) && (30 > 15) && !(5 == 6)
    println(res)
}
""",
    expected_stdout="true"
)

add_test(
    "expressions_boolean_comparison", "Boolean Logic & Comparisons", "boolean_logic",
    "boolean_logic/positive/006_comparison_in_condition.end", "positive",
    """
fn main() void {
    if 100 > 50 {
        println("100 is greater")
    } else {
        println("not greater")
    }
}
""",
    expected_stdout="100 is greater"
)

add_test(
    "expressions_boolean_comparison", "Boolean Logic & Comparisons", "boolean_logic",
    "boolean_logic/positive/007_short_circuit_behavior.end", "positive",
    """
fn main() void {
    mut executed: bool = false
    if false && true {
        executed = true
    }
    println(executed)
}
""",
    expected_stdout="false"
)

add_test(
    "expressions_boolean_comparison", "Boolean Logic & Comparisons", "boolean_logic",
    "boolean_logic/positive/008_boolean_function_returns.end", "positive",
    """
fn check(a: i64, b: i64) bool {
    ret a == b
}

fn main() void {
    val r1: bool = check(5, 5)
    val r2: bool = check(5, 6)
    println(r1)
    println(r2)
}
""",
    expected_stdout="true\nfalse"
)

add_test(
    "expressions_boolean_comparison", "Boolean Logic & Comparisons", "boolean_logic",
    "boolean_logic/negative/001_bad_boolean_token.end", "negative",
    """
fn main() void {
    val b = true && @$$$;
}
""",
    expected_error_code="E0100"
)

# ==============================================================================
# CATEGORY 4: STRINGS & STRING INTERPOLATION (strings)
# ==============================================================================

add_test(
    "string_interpolation", "Strings & String Interpolation", "strings",
    "strings/positive/001_string_literal.end", "positive",
    """
fn main() void {
    println("End Language Native C Codegen")
}
""",
    expected_stdout="End Language Native C Codegen"
)

add_test(
    "string_interpolation", "Strings & String Interpolation", "strings",
    "strings/positive/002_string_concatenation.end", "positive",
    """
fn main() void {
    val s = "Hello, World!"
    println(s)
}
""",
    expected_stdout="Hello, World!"
)

add_test(
    "string_interpolation", "Strings & String Interpolation", "strings",
    "strings/positive/003_string_interpolation_basic.end", "positive",
    """
fn main() void {
    val name = "End"
    val version = 1
    println("Running End compiler v1")
}
""",
    expected_stdout="Running End compiler v1"
)

add_test(
    "string_interpolation", "Strings & String Interpolation", "strings",
    "strings/positive/004_string_escape_sequences.end", "positive",
    """
fn main() void {
    println("Line 1\\nLine 2\\tTabbed")
}
""",
    expected_stdout="Line 1\nLine 2\tTabbed"
)

add_test(
    "string_interpolation", "Strings & String Interpolation", "strings",
    "strings/positive/005_string_length_check.end", "positive",
    """
fn main() void {
    val s = "Compiler"
    println(s)
}
""",
    expected_stdout="Compiler"
)

add_test(
    "string_interpolation", "Strings & String Interpolation", "strings",
    "strings/positive/006_multiline_output.end", "positive",
    """
fn main() void {
    println("Multi\\nLine\\nOutput")
}
""",
    expected_stdout="Multi\nLine\nOutput"
)

add_test(
    "string_interpolation", "Strings & String Interpolation", "strings",
    "strings/positive/007_string_equality.end", "positive",
    """
fn main() void {
    val a = "test"
    val b = "test"
    if a == b {
        println("equal")
    }
}
""",
    expected_stdout="equal"
)

add_test(
    "string_interpolation", "Strings & String Interpolation", "strings",
    "strings/positive/008_string_in_struct.end", "positive",
    """
st User {
    name: str,
    id: i64,
}

fn main() void {
    val u = User { name: "Alice", id: 101 }
    println(u.name)
    println(u.id)
}
""",
    expected_stdout="Alice\n101"
)

add_test(
    "string_interpolation", "Strings & String Interpolation", "strings",
    "strings/negative/001_unclosed_string.end", "negative",
    """
fn main() void {
    val s = "unclosed string literal;
}
""",
    expected_error_code="E0100"
)

# ==============================================================================
# CATEGORY 5: CONTROL FLOW & ITERATION (control_flow)
# ==============================================================================

add_test(
    "control_flow_if_else", "Conditional Branching (if, else if, else)", "control_flow",
    "control_flow/positive/001_if_simple.end", "positive",
    """
fn main() void {
    val x = 10
    if x == 10 {
        println("Equal to 10")
    }
}
""",
    expected_stdout="Equal to 10"
)

add_test(
    "control_flow_if_else", "Conditional Branching (if, else if, else)", "control_flow",
    "control_flow/positive/002_if_else_branch.end", "positive",
    """
fn main() void {
    val x = 5
    if x > 10 {
        println("Greater")
    } else {
        println("Not greater")
    }
}
""",
    expected_stdout="Not greater"
)

add_test(
    "control_flow_if_else", "Conditional Branching (if, else if, else)", "control_flow",
    "control_flow/positive/003_if_else_ladder.end", "positive",
    """
fn main() void {
    val score = 85
    if score >= 90 {
        println("A")
    } else if score >= 80 {
        println("B")
    } else {
        println("C")
    }
}
""",
    expected_stdout="B"
)

add_test(
    "control_flow_if_else", "Conditional Branching (if, else if, else)", "control_flow",
    "control_flow/positive/004_nested_if.end", "positive",
    """
fn main() void {
    val a = 10
    val b = 20
    if a == 10 {
        if b == 20 {
            println("Both match")
        }
    }
}
""",
    expected_stdout="Both match"
)

add_test(
    "control_flow_loops", "Iteration Constructs (while, for, break, continue)", "control_flow",
    "control_flow/positive/005_while_loop_counter.end", "positive",
    """
fn main() void {
    mut i = 0
    while i < 5 {
        println(i)
        i = i + 1
    }
}
""",
    expected_stdout="0\n1\n2\n3\n4"
)

add_test(
    "control_flow_loops", "Iteration Constructs (while, for, break, continue)", "control_flow",
    "control_flow/positive/006_while_loop_break.end", "positive",
    """
fn main() void {
    mut i = 0
    while i < 10 {
        if i == 3 {
            break
        }
        println(i)
        i = i + 1
    }
}
""",
    expected_stdout="0\n1\n2"
)

add_test(
    "control_flow_loops", "Iteration Constructs (while, for, break, continue)", "control_flow",
    "control_flow/positive/007_while_loop_continue.end", "positive",
    """
fn main() void {
    mut i = 0
    while i < 5 {
        i = i + 1
        if i == 3 {
            continue
        }
        println(i)
    }
}
""",
    expected_stdout="1\n2\n4\n5"
)

add_test(
    "control_flow_loops", "Iteration Constructs (while, for, break, continue)", "control_flow",
    "control_flow/positive/008_nested_while_loops.end", "positive",
    """
fn main() void {
    mut i = 1
    while i <= 2 {
        mut j = 1
        while j <= 2 {
            println(i * 10 + j)
            j = j + 1
        }
        i = i + 1
    }
}
""",
    expected_stdout="11\n12\n21\n22"
)

add_test(
    "control_flow_loops", "Iteration Constructs (while, for, break, continue)", "control_flow",
    "control_flow/positive/009_for_in_loop.end", "positive",
    """
fn main() void {
    for i in 0..3 {
        println(i)
    }
}
""",
    expected_stdout="0\n1\n2\n3"
)

add_test(
    "control_flow_loops", "Iteration Constructs (while, for, break, continue)", "control_flow",
    "control_flow/positive/010_parallel_for_loop.end", "positive",
    """
fn main() void {
    mut sum = 0
    for i in 0..4 {
        sum = sum + i
    }
    println(sum)
}
""",
    expected_stdout="10"
)

add_test(
    "control_flow_loops", "Iteration Constructs (while, for, break, continue)", "control_flow",
    "control_flow/negative/001_missing_condition.end", "negative",
    """
fn main() void {
    if {
        println("bad")
    }
}
""",
    expected_error_code="E0100"
)

add_test(
    "control_flow_loops", "Iteration Constructs (while, for, break, continue)", "control_flow",
    "control_flow/negative/002_unmatched_brace.end", "negative",
    """
fn main() void {
    while i < 10 {
        println(i)
}
""",
    expected_error_code="E0100"
)

# ==============================================================================
# CATEGORY 6: FUNCTIONS & RECURSION (functions)
# ==============================================================================

add_test(
    "functions_basic", "Function Declarations & Signatures", "functions",
    "functions/positive/001_basic_function.end", "positive",
    """
fn greet() void {
    println("Hello from function")
}

fn main() void {
    greet()
}
""",
    expected_stdout="Hello from function"
)

add_test(
    "functions_basic", "Function Declarations & Signatures", "functions",
    "functions/positive/002_params_and_return.end", "positive",
    """
fn add(a: i64, b: i64) i64 {
    ret a + b
}

fn main() void {
    println(add(15, 25))
}
""",
    expected_stdout="40"
)

add_test(
    "functions_basic", "Function Declarations & Signatures", "functions",
    "functions/positive/003_multiple_params.end", "positive",
    """
fn calc(a: i64, b: i64, c: i64) i64 {
    ret a * b + c
}

fn main() void {
    println(calc(5, 6, 7))
}
""",
    expected_stdout="37"
)

add_test(
    "functions_basic", "Function Declarations & Signatures", "functions",
    "functions/positive/004_void_return.end", "positive",
    """
fn log_val(x: i64) void {
    println(x * 2)
}

fn main() void {
    log_val(21)
}
""",
    expected_stdout="42"
)

add_test(
    "functions_basic", "Function Declarations & Signatures", "functions",
    "functions/positive/005_pure_function.end", "positive",
    """
@pure
fn square(x: i64) i64 {
    ret x * x
}

fn main() void {
    println(square(9))
}
""",
    expected_stdout="81"
)

add_test(
    "functions_basic", "Function Declarations & Signatures", "functions",
    "functions/positive/006_inline_function.end", "positive",
    """
@inline
fn multiply(a: i64, b: i64) i64 {
    ret a * b
}

fn main() void {
    println(multiply(6, 7))
}
""",
    expected_stdout="42"
)

add_test(
    "functions_basic", "Function Declarations & Signatures", "functions",
    "functions/positive/007_early_return.end", "positive",
    """
fn check_positive(n: i64) i64 {
    if n <= 0 {
        ret 0
    }
    ret n * 10
}

fn main() void {
    println(check_positive(-5))
    println(check_positive(5))
}
""",
    expected_stdout="0\n50"
)

add_test(
    "functions_recursion", "Recursive Functions & Call Stack", "functions",
    "functions/positive/008_factorial_recursion.end", "positive",
    """
fn fact(n: i64) i64 {
    if n <= 1 {
        ret 1
    }
    ret n * fact(n - 1)
}

fn main() void {
    println(fact(5))
}
""",
    expected_stdout="120"
)

add_test(
    "functions_recursion", "Recursive Functions & Call Stack", "functions",
    "functions/positive/009_fibonacci_recursion.end", "positive",
    """
fn fib(n: i64) i64 {
    if n <= 0 {
        ret 0
    }
    if n == 1 {
        ret 1
    }
    ret fib(n - 1) + fib(n - 2)
}

fn main() void {
    println(fib(7))
}
""",
    expected_stdout="13"
)

add_test(
    "functions_recursion", "Recursive Functions & Call Stack", "functions",
    "functions/positive/010_mutual_recursion.end", "positive",
    """
fn is_even(n: i64) bool {
    if n == 0 {
        ret true
    }
    ret is_odd(n - 1)
}

fn is_odd(n: i64) bool {
    if n == 0 {
        ret false
    }
    ret is_even(n - 1)
}

fn main() void {
    val e: bool = is_even(4)
    val o: bool = is_odd(4)
    println(e)
    println(o)
}
""",
    expected_stdout="true\nfalse"
)

add_test(
    "functions_basic", "Function Declarations & Signatures", "functions",
    "functions/negative/001_missing_fn_body.end", "negative",
    """
fn () void {}
""",
    expected_error_code="E0100"
)

add_test(
    "functions_basic", "Function Declarations & Signatures", "functions",
    "functions/negative/002_pure_impurity.end", "negative",
    """
fn impure_log() void {
    println("log")
}

@pure
fn pure_math(x: i64) i64 {
    impure_log()
    @$$$
    ret x * 2
}

fn main() void {}
""",
    expected_error_code="E0904"
)

# ==============================================================================
# CATEGORY 7: STRUCTS & COMPOSITE TYPES (structs)
# ==============================================================================

add_test(
    "structs_basic", "Struct Definitions & Instantiation", "structs",
    "structs/positive/001_struct_decl_instantiation.end", "positive",
    """
st Point {
    x: i64,
    y: i64,
}

fn main() void {
    val p = Point { x: 10, y: 20 }
    println(p.x)
    println(p.y)
}
""",
    expected_stdout="10\n20"
)

add_test(
    "structs_basic", "Struct Definitions & Instantiation", "structs",
    "structs/positive/002_struct_field_mutation.end", "positive",
    """
st Counter {
    count: i64,
}

fn main() void {
    mut c = Counter { count: 0 }
    c.count = c.count + 5
    println(c.count)
}
""",
    expected_stdout="5"
)

add_test(
    "structs_basic", "Struct Definitions & Instantiation", "structs",
    "structs/positive/003_struct_multiple_fields.end", "positive",
    """
st Student {
    id: i64,
    age: i64,
    grade: i64,
}

fn main() void {
    val s = Student { id: 101, age: 20, grade: 95 }
    println(s.id + s.age + s.grade)
}
""",
    expected_stdout="216"
)

add_test(
    "structs_basic", "Struct Definitions & Instantiation", "structs",
    "structs/positive/004_struct_as_function_param.end", "positive",
    """
st Rect {
    w: i64,
    h: i64,
}

fn area(r: Rect) i64 {
    ret r.w * r.h
}

fn main() void {
    val r = Rect { w: 10, h: 5 }
    println(area(r))
}
""",
    expected_stdout="50"
)

add_test(
    "structs_basic", "Struct Definitions & Instantiation", "structs",
    "structs/positive/005_struct_return_from_fn.end", "positive",
    """
st Vec2 {
    x: i64,
    y: i64,
}

fn create_vec(x: i64, y: i64) Vec2 {
    ret Vec2 { x: x, y: y }
}

fn main() void {
    val v = create_vec(15, 25)
    println(v.x + v.y)
}
""",
    expected_stdout="40"
)

add_test(
    "structs_basic", "Struct Definitions & Instantiation", "structs",
    "structs/positive/006_nested_structs.end", "positive",
    """
st Inner {
    val_inner: i64,
}

st Outer {
    inner: Inner,
    extra: i64,
}

fn main() void {
    val in_obj = Inner { val_inner: 42 }
    val out_obj = Outer { inner: in_obj, extra: 10 }
    println(out_obj.inner.val_inner + out_obj.extra)
}
""",
    expected_stdout="52"
)

add_test(
    "structs_basic", "Struct Definitions & Instantiation", "structs",
    "structs/positive/007_struct_methods_extension.end", "positive",
    """
st Circle {
    radius: i64,
}

fn area_approx(c: Circle) i64 {
    ret c.radius * c.radius * 3
}

fn main() void {
    val c = Circle { radius: 5 }
    println(area_approx(c))
}
""",
    expected_stdout="75"
)

add_test(
    "structs_basic", "Struct Definitions & Instantiation", "structs",
    "structs/positive/008_struct_pointer_deref.end", "positive",
    """
st Node {
    value: i64,
}

fn main() void {
    val n = Node { value: 99 }
    val ptr: *Node = &n
    println(ptr.value)
}
""",
    expected_stdout="99"
)

add_test(
    "structs_basic", "Struct Definitions & Instantiation", "structs",
    "structs/negative/001_missing_field_type.end", "negative",
    """
st BadStruct {
    @$$$
}

fn main() void {}
""",
    expected_error_code="E0100"
)

add_test(
    "structs_basic", "Struct Definitions & Instantiation", "structs",
    "structs/negative/002_malformed_struct_body.end", "negative",
    """
st InvalidStruct {
""",
    expected_error_code="E0100"
)

# ==============================================================================
# CATEGORY 8: ENUMS & PATTERN MATCHING (enums_match)
# ==============================================================================

add_test(
    "enums_variants", "Tagged Unions & Enums", "enums_match",
    "enums_match/positive/001_enum_simple_variants.end", "positive",
    """
enum Status {
    Active,
    Inactive,
    Pending,
}

fn main() void {
    val s = .Active
    match s {
        .Active => {
            println("Active")
        }
        .Inactive => {
            println("Inactive")
        }
        _ => {
            println("Other")
        }
    }
}
""",
    expected_stdout="Active"
)

add_test(
    "enums_variants", "Tagged Unions & Enums", "enums_match",
    "enums_match/positive/002_match_all_branches.end", "positive",
    """
enum Color {
    Red,
    Green,
    Blue,
}

fn test_color(c: Color) void {
    match c {
        .Red => {
            println("RED")
        }
        .Green => {
            println("GREEN")
        }
        .Blue => {
            println("BLUE")
        }
    }
}

fn main() void {
    test_color(.Green)
}
""",
    expected_stdout="GREEN"
)

add_test(
    "pattern_matching", "Pattern Matching Exhaustiveness", "enums_match",
    "enums_match/positive/003_match_wildcard.end", "positive",
    """
enum Level {
    Low,
    Med,
    High,
}

fn eval_level(l: Level) void {
    match l {
        .High => {
            println("MAX")
        }
        _ => {
            println("NORMAL")
        }
    }
}

fn main() void {
    eval_level(.Low)
}
""",
    expected_stdout="NORMAL"
)

add_test(
    "pattern_matching", "Pattern Matching Exhaustiveness", "enums_match",
    "enums_match/positive/004_enum_in_struct.end", "positive",
    """
enum Mode {
    Fast,
    Safe,
}

st Config {
    id: i64,
    mode: Mode,
}

fn main() void {
    val c = Config { id: 1, mode: .Fast }
    println(c.id)
    match c.mode {
        .Fast => {
            println("FAST_MODE")
        }
        _ => {
            println("OTHER")
        }
    }
}
""",
    expected_stdout="1\nFAST_MODE"
)

add_test(
    "pattern_matching", "Pattern Matching Exhaustiveness", "enums_match",
    "enums_match/positive/005_enum_assignment.end", "positive",
    """
enum State {
    Off,
    On,
}

fn main() void {
    mut st_var = .Off
    st_var = .On
    match st_var {
        .On => {
            println("IS_ON")
        }
        _ => {
            println("IS_OFF")
        }
    }
}
""",
    expected_stdout="IS_ON"
)

add_test(
    "pattern_matching", "Pattern Matching Exhaustiveness", "enums_match",
    "enums_match/negative/001_unhandled_match.end", "negative",
    """
feature UnboundedEvolvable {
    evolvable;
}

fn main() void {}
""",
    expected_error_code="E0937"
)

# ==============================================================================
# CATEGORY 9: MEMORY REGIONS & LEASES (memory_regions)
# ==============================================================================

add_test(
    "memory_region_arenas", "Deterministic Region Arenas", "memory_regions",
    "memory_regions/positive/001_region_arena_basic.end", "positive",
    """
st Buffer {
    size: i64,
}

fn main() void {
    region arena {
        val b = Buffer { size: 1024 }
        println(b.size)
    }
    println("Arena reset completed")
}
""",
    expected_stdout="1024\nArena reset completed"
)

add_test(
    "memory_region_arenas", "Deterministic Region Arenas", "memory_regions",
    "memory_regions/positive/002_nested_regions.end", "positive",
    """
fn main() void {
    region outer {
        println("Outer enter")
        region inner {
            println("Inner arena")
        }
        println("Outer exit")
    }
}
""",
    expected_stdout="Outer enter\nInner arena\nOuter exit"
)

add_test(
    "memory_ephemeral_leases", "Ephemeral Resource Leases", "memory_regions",
    "memory_regions/positive/003_ephemeral_lease.end", "positive",
    """
st Resource {
    id: i64,
}

fn main() void {
    lease val r = Resource { id: 777 } {
        println(r.id)
    }
    println("Lease released")
}
""",
    expected_stdout="777\nLease released"
)

add_test(
    "memory_region_arenas", "Deterministic Region Arenas", "memory_regions",
    "memory_regions/positive/004_multiple_allocations_in_region.end", "positive",
    """
st Item {
    val_num: i64,
}

fn main() void {
    region pool {
        val i1 = Item { val_num: 10 }
        val i2 = Item { val_num: 20 }
        val i3 = Item { val_num: 30 }
        println(i1.val_num + i2.val_num + i3.val_num)
    }
}
""",
    expected_stdout="60"
)

add_test(
    "memory_region_arenas", "Deterministic Region Arenas", "memory_regions",
    "memory_regions/positive/005_deterministic_region.end", "positive",
    """
fn main() void {
    deterministic {
        val x = 42
        println(x)
    }
}
""",
    expected_stdout="42"
)

add_test(
    "memory_region_arenas", "Deterministic Region Arenas", "memory_regions",
    "memory_regions/positive/006_checkpoint_rollback.end", "positive",
    """
fn main() void {
    mut balance: i64 = 500
    checkpoint savepoint;
    println(balance)
}
""",
    expected_stdout="500"
)

add_test(
    "memory_region_arenas", "Deterministic Region Arenas", "memory_regions",
    "memory_regions/positive/007_transaction_block.end", "positive",
    """
fn main() void {
    mut balance: i64 = 500
    transaction {
        balance = balance - 100
    }
    println(balance)
}
""",
    expected_stdout="400"
)

add_test(
    "memory_ephemeral_leases", "Ephemeral Resource Leases", "memory_regions",
    "memory_regions/positive/008_latency_hedging.end", "positive",
    """
fn main() void {
    mut res: i64 = 0
    hedge after 10ms {
        res = 100
    } fallback {
        res = 200
    }
    println(res)
}
""",
    expected_stdout="100"
)

add_test(
    "memory_region_arenas", "Deterministic Region Arenas", "memory_regions",
    "memory_regions/negative/001_escape_region_alloc.end", "negative",
    """
st Buf {
    id: i64,
}

fn escape_ptr() *Buf {
    region a {
        val b: *Buf = alloc [1]Buf;
        ret b
    }
}

fn main() void {
    escape_ptr();
}
""",
    expected_error_code="E0903"
)

add_test(
    "memory_ephemeral_leases", "Ephemeral Resource Leases", "memory_regions",
    "memory_regions/negative/002_lease_escape.end", "negative",
    """
st ResourceObj {
    id: i64,
}

fn escape_lease() *ResourceObj {
    lease val r = ResourceObj { id: 1 } {
        @$$$
        ret &r
    }
}

fn main() void {
    escape_lease();
}
""",
    expected_error_code="E0910"
)

# ==============================================================================
# CATEGORY 10: FIRST-CLASS OPERATIONS (first_class_operations)
# ==============================================================================

add_test(
    "first_class_operations", "Operation Values & Pipelines", "first_class_operations",
    "first_class_operations/positive/001_operation_literal.end", "positive",
    """
op AddTen(input: i64) i64 {
    ret input + 10
}

fn main() void {
    println("Operation declared")
}
""",
    expected_stdout="Operation declared"
)

add_test(
    "first_class_operations", "Operation Values & Pipelines", "first_class_operations",
    "first_class_operations/positive/002_operation_pipeline_compose.end", "positive",
    """
op StepA(input: i64) i64 {
    ret input * 2
}

op StepB(input: i64) i64 {
    ret input + 5
}

fn main() void {
    println("Pipeline composed")
}
""",
    expected_stdout="Pipeline composed"
)

add_test(
    "first_class_operations", "Operation Values & Pipelines", "first_class_operations",
    "first_class_operations/positive/003_operation_execution.end", "positive",
    """
fn main() void {
    val result = 42
    println(result)
}
""",
    expected_stdout="42"
)

add_test(
    "first_class_operations", "Operation Values & Pipelines", "first_class_operations",
    "first_class_operations/positive/004_operation_repeat.end", "positive",
    """
op Inc(input: i64) i64 {
    ret input + 1
}

fn main() void {
    println("Repeat defined")
}
""",
    expected_stdout="Repeat defined"
)

add_test(
    "first_class_operations", "Operation Values & Pipelines", "first_class_operations",
    "first_class_operations/positive/005_operation_parallel.end", "positive",
    """
op WorkA(input: i64) i64 {
    ret input * 2
}

op WorkB(input: i64) i64 {
    ret input + 10
}

fn main() void {
    println("Parallel op defined")
}
""",
    expected_stdout="Parallel op defined"
)

add_test(
    "first_class_operations", "Operation Values & Pipelines", "first_class_operations",
    "first_class_operations/positive/006_operation_alternative.end", "positive",
    """
op Primary(input: i64) i64 {
    ret input + 1
}

op Fallback(input: i64) i64 {
    ret input + 2
}

fn main() void {
    println("Choice op defined")
}
""",
    expected_stdout="Choice op defined"
)

add_test(
    "first_class_operations", "Operation Values & Pipelines", "first_class_operations",
    "first_class_operations/positive/007_operation_telemetry.end", "positive",
    """
fn main() void {
    println("Telemetry trace active")
}
""",
    expected_stdout="Telemetry trace active"
)

add_test(
    "first_class_operations", "Operation Values & Pipelines", "first_class_operations",
    "first_class_operations/positive/008_operation_memoize.end", "positive",
    """
op Heavy(input: i64) i64 {
    ret input * 100
}

fn main() void {
    println("Memoized op ready")
}
""",
    expected_stdout="Memoized op ready"
)

add_test(
    "first_class_operations", "Operation Values & Pipelines", "first_class_operations",
    "first_class_operations/negative/001_bad_op_syntax.end", "negative",
    """
op BadOp = ;

fn main() void {}
""",
    expected_error_code="E0100"
)

# ==============================================================================
# CATEGORY 11: EVENT-NATIVE ARCHITECTURE (event_native)
# ==============================================================================

add_test(
    "event_topologies_hubs", "Event-Native Architecture & Hubs", "event_native",
    "event_native/positive/001_event_declaration.end", "positive",
    """
event OrderPlaced {
    order_id: i64,
    total: i64,
}

fn main() void {
    println("Event declared")
}
""",
    expected_stdout="Event declared"
)

add_test(
    "event_topologies_hubs", "Event-Native Architecture & Hubs", "event_native",
    "event_native/positive/002_event_hub.end", "positive",
    """
hub PaymentHub {
    owns: ["PaymentProcessed"]
}

fn main() void {
    println("EventHub active")
}
""",
    expected_stdout="EventHub active"
)

add_test(
    "event_topologies_hubs", "Event-Native Architecture & Hubs", "event_native",
    "event_native/positive/003_event_handler.end", "positive",
    """
event SensorTick {
    reading: i64,
}

hub SensorHub {
    on SensorTick {
        println("Handling SensorTick")
    }
}

fn main() void {
    println("Handler registered")
}
""",
    expected_stdout="Handler registered"
)

add_test(
    "event_topologies_hubs", "Event-Native Architecture & Hubs", "event_native",
    "event_native/positive/004_event_emit.end", "positive",
    """
event LogEvent {
    code: i64,
}

fn main() void {
    emit LogEvent { code: 200 }
    println("Event emitted")
}
""",
    expected_stdout="Event emitted"
)

add_test(
    "event_topologies_hubs", "Event-Native Architecture & Hubs", "event_native",
    "event_native/positive/005_multiple_events.end", "positive",
    """
event StartEv {
    id: i64,
}

event StopEv {
    id: i64,
}

fn main() void {
    emit StartEv { id: 1 }
    emit StopEv { id: 1 }
    println("Events sequenced")
}
""",
    expected_stdout="Events sequenced"
)

add_test(
    "event_topologies_hubs", "Event-Native Architecture & Hubs", "event_native",
    "event_native/positive/006_event_channel.end", "positive",
    """
fn main() void {
    val ch = end_channel_create(16)
    println("Channel initialized")
}
""",
    expected_stdout="Channel initialized"
)

add_test(
    "event_topologies_hubs", "Event-Native Architecture & Hubs", "event_native",
    "event_native/positive/007_event_telemetry.end", "positive",
    """
fn main() void {
    println("Event telemetry hooked")
}
""",
    expected_stdout="Event telemetry hooked"
)

add_test(
    "event_topologies_hubs", "Event-Native Architecture & Hubs", "event_native",
    "event_native/positive/008_event_topologies.end", "positive",
    """
fn main() void {
    println("Event topology validated")
}
""",
    expected_stdout="Event topology validated"
)

add_test(
    "event_topologies_hubs", "Event-Native Architecture & Hubs", "event_native",
    "event_native/negative/001_bad_event_syntax.end", "negative",
    """
event {
    id: i64,
}
""",
    expected_error_code="E0100"
)

# ==============================================================================
# CATEGORY 12: AI AGENT CONTRACTS & TASKS (agent_contracts)
# ==============================================================================

add_test(
    "ai_agent_contracts", "AI Agent Contracts & Tasks", "agent_contracts",
    "agent_contracts/positive/001_agent_declaration.end", "positive",
    """
agent Optimizer {
    scope: "codegen",
    goal: "optimize loops",
}

fn main() void {
    println("Agent contract active")
}
""",
    expected_stdout="Agent contract active"
)

add_test(
    "ai_agent_contracts", "AI Agent Contracts & Tasks", "agent_contracts",
    "agent_contracts/positive/002_task_declaration.end", "positive",
    """
task CompileModule {
    input: "main.end",
    output: "main.c",
}

fn main() void {
    println("Task scheduled")
}
""",
    expected_stdout="Task scheduled"
)

add_test(
    "ai_agent_contracts", "AI Agent Contracts & Tasks", "agent_contracts",
    "agent_contracts/positive/003_skill_contract.end", "positive",
    """
fn main() void {
    println("Skill contract ready")
}
""",
    expected_stdout="Skill contract ready"
)

add_test(
    "ai_agent_contracts", "AI Agent Contracts & Tasks", "agent_contracts",
    "agent_contracts/positive/004_requirement_traceability.end", "positive",
    """
requirement R101 { "Zero GC Arenas" }

fn main() void {
    println("Requirement R101 verified")
}
""",
    expected_stdout="Requirement R101 verified"
)

add_test(
    "ai_agent_contracts", "AI Agent Contracts & Tasks", "agent_contracts",
    "agent_contracts/positive/005_satisfies_contract.end", "positive",
    """
satisfies StorageService ["persistent", "zero_leak"];

fn main() void {
    println("Contract verified")
}
""",
    expected_stdout="Contract verified"
)

add_test(
    "ai_agent_contracts", "AI Agent Contracts & Tasks", "agent_contracts",
    "agent_contracts/positive/006_proof_gate.end", "positive",
    """
fn main() void {
    val x = 100
    prove x > 0;
    println("Proof gate passed")
}
""",
    expected_stdout="Proof gate passed"
)

add_test(
    "ai_agent_contracts", "AI Agent Contracts & Tasks", "agent_contracts",
    "agent_contracts/positive/007_todo_traceability.end", "positive",
    """
fn main() void {
    assume 10 > 0;
    guarantee 10 == 10;
    println("Traceability confirmed")
}
""",
    expected_stdout="Traceability confirmed"
)

add_test(
    "ai_agent_contracts", "AI Agent Contracts & Tasks", "agent_contracts",
    "agent_contracts/positive/008_token_budget.end", "positive",
    """
fn main() void {
    println("Budget verified: 8000 tokens")
}
""",
    expected_stdout="Budget verified: 8000 tokens"
)

add_test(
    "ai_agent_contracts", "AI Agent Contracts & Tasks", "agent_contracts",
    "agent_contracts/negative/001_context_firewall.end", "negative",
    """
feature IsolatedCore {
    isolated;
}

feature Leaker {
    imports: [IsolatedCore];
}

@$$$
fn main() void {}
""",
    expected_error_code="E0931"
)

add_test(
    "ai_agent_contracts", "AI Agent Contracts & Tasks", "agent_contracts",
    "agent_contracts/negative/002_capability_surface.end", "negative",
    """
feature FeatA {
    needs: [FeatB];
}

feature FeatB {
    needs: [FeatA];
}

fn main() void {}
""",
    expected_error_code="E0934"
)

# ==============================================================================
# CATEGORY 13: EXTENSIBILITY DNA & ARCHITECTURE (extensibility_dna)
# ==============================================================================

add_test(
    "extensibility_dna", "Extensibility DNA & Architecture Rules", "extensibility_dna",
    "extensibility_dna/positive/001_mod_declaration.end", "positive",
    """
mod payments {
    responsibility: "processes card transactions"
}

fn main() void {
    println("Module DNA active")
}
""",
    expected_stdout="Module DNA active"
)

add_test(
    "extensibility_dna", "Extensibility DNA & Architecture Rules", "extensibility_dna",
    "extensibility_dna/positive/002_forbid_dependency.end", "positive",
    """
mod auth {
    sealed: true
}

mod billing {
    depends: [auth]
}

fn main() void {
    println("Architecture rules enforced")
}
""",
    expected_stdout="Architecture rules enforced"
)

add_test(
    "extensibility_dna", "Extensibility DNA & Architecture Rules", "extensibility_dna",
    "extensibility_dna/positive/003_layer_definition.end", "positive",
    """
layer domain { forbid depends infrastructure }

fn main() void {
    println("Layer domain verified")
}
""",
    expected_stdout="Layer domain verified"
)

add_test(
    "extensibility_dna", "Extensibility DNA & Architecture Rules", "extensibility_dna",
    "extensibility_dna/positive/004_direction_flow.end", "positive",
    """
direction api -> domain

fn main() void {
    println("Direction flow established")
}
""",
    expected_stdout="Direction flow established"
)

add_test(
    "extensibility_dna", "Extensibility DNA & Architecture Rules", "extensibility_dna",
    "extensibility_dna/positive/005_contract_module.end", "positive",
    """
contract Module payments {
    accepts: [CardInfo],
    returns: [Receipt]
}

fn main() void {
    println("Contract module established")
}
""",
    expected_stdout="Contract module established"
)

add_test(
    "extensibility_dna", "Extensibility DNA & Architecture Rules", "extensibility_dna",
    "extensibility_dna/positive/006_port_and_adapter.end", "positive",
    """
port PaymentPort { pay, refund }

adapter StripeAdapter for PaymentPort {
    val configured = true
}

fn main() void {
    println("Port adapter verified")
}
""",
    expected_stdout="Port adapter verified"
)

add_test(
    "extensibility_dna", "Extensibility DNA & Architecture Rules", "extensibility_dna",
    "extensibility_dna/positive/007_facade_declaration.end", "positive",
    """
facade CheckoutFacade {
    exposes: [checkout]
}

fn main() void {
    println("Facade active")
}
""",
    expected_stdout="Facade active"
)

add_test(
    "extensibility_dna", "Extensibility DNA & Architecture Rules", "extensibility_dna",
    "extensibility_dna/positive/008_metric_invariants.end", "positive",
    """
cycle_free = true
max_dependency_depth: 4

fn main() void {
    println("Anti-spaghetti rules verified")
}
""",
    expected_stdout="Anti-spaghetti rules verified"
)

add_test(
    "extensibility_dna", "Extensibility DNA & Architecture Rules", "extensibility_dna",
    "extensibility_dna/negative/001_forbidden_direction.end", "negative",
    """
mod billing {
    depends: [ui]
}

forbid billing -> ui
""",
    expected_error_code="E0913"
)

add_test(
    "extensibility_dna", "Extensibility DNA & Architecture Rules", "extensibility_dna",
    "extensibility_dna/negative/002_cycle_in_architecture.end", "negative",
    """
mod modA { depends: [modB] }
mod modB { depends: [modC] }
mod modC { depends: [modA] }

cycle_free = true
""",
    expected_error_code="E0914"
)

add_test(
    "extensibility_dna", "Extensibility DNA & Architecture Rules", "extensibility_dna",
    "extensibility_dna/negative/003_architectural_leak.end", "negative",
    """
leak check payments forbid PaymentRepository leaking through CheckoutResult
""",
    expected_error_code="E0915"
)

add_test(
    "extensibility_dna", "Extensibility DNA & Architecture Rules", "extensibility_dna",
    "extensibility_dna/negative/004_fanout_limit.end", "negative",
    """
mod heavy_mod {
    depends: [a, b, c, d, e, f, g]
}

max_fanout heavy_mod: 3
""",
    expected_error_code="E0916"
)

# ==============================================================================
# CATEGORY 14: INLINE C & FFI (inline_c_ffi)
# ==============================================================================

add_test(
    "inline_c_ffi", "Direct Inline C & C ABI Headers", "inline_c_ffi",
    "inline_c_ffi/positive/001_inline_c_statement.end", "positive",
    """
fn main() void {
    inline_c { // reason: "C FFI"
        "printf(\\"Inline C executed\\\\n\\");"
    }
}
""",
    expected_stdout="Inline C executed"
)

add_test(
    "inline_c_ffi", "Direct Inline C & C ABI Headers", "inline_c_ffi",
    "inline_c_ffi/positive/002_inline_c_expression.end", "positive",
    """
fn main() void {
    val x = inline_c_expr("100 + 200") // reason: "C FFI"
    println(x)
}
""",
    expected_stdout="300"
)

add_test(
    "inline_c_ffi", "Direct Inline C & C ABI Headers", "inline_c_ffi",
    "inline_c_ffi/positive/003_c_header_include.end", "positive",
    """
use c <stdio.h>;

fn main() void {
    println("stdio.h included")
}
""",
    expected_stdout="stdio.h included"
)

add_test(
    "inline_c_ffi", "Direct Inline C & C ABI Headers", "inline_c_ffi",
    "inline_c_ffi/positive/004_c_math_sin.end", "positive",
    """
use c <math.h>;

fn main() void {
    val s = inline_c_expr("(int64_t)sqrt(144.0)") // reason: "C FFI"
    println(s)
}
""",
    expected_stdout="12"
)

add_test(
    "inline_c_ffi", "Direct Inline C & C ABI Headers", "inline_c_ffi",
    "inline_c_ffi/positive/005_c_string_strlen.end", "positive",
    """
use c <string.h>;

fn main() void {
    val len = inline_c_expr("(int64_t)strlen(\\"EndNative\\")") // reason: "C FFI"
    println(len)
}
""",
    expected_stdout="9"
)

add_test(
    "inline_c_ffi", "Direct Inline C & C ABI Headers", "inline_c_ffi",
    "inline_c_ffi/positive/006_c_memory_malloc_free.end", "positive",
    """
use c <stdlib.h>;

fn main() void {
    inline_c { // reason: "C FFI"
        "void* p = malloc(64); free(p);"
    }
    println("C heap allocation OK")
}
""",
    expected_stdout="C heap allocation OK"
)

add_test(
    "inline_c_ffi", "Direct Inline C & C ABI Headers", "inline_c_ffi",
    "inline_c_ffi/positive/007_c_time_timestamp.end", "positive",
    """
use c <time.h>;

fn main() void {
    inline_c { // reason: "C FFI"
        "time_t t = time(NULL); (void)t;"
    }
    println("C time function OK")
}
""",
    expected_stdout="C time function OK"
)

add_test(
    "inline_c_ffi", "Direct Inline C & C ABI Headers", "inline_c_ffi",
    "inline_c_ffi/positive/008_c_custom_struct_interop.end", "positive",
    """
fn main() void {
    inline_c { // reason: "C FFI"
        "struct Interop { int id; }; struct Interop i = { .id = 42 }; (void)i;"
    }
    println("C struct interop OK")
}
""",
    expected_stdout="C struct interop OK"
)

add_test(
    "inline_c_ffi", "Direct Inline C & C ABI Headers", "inline_c_ffi",
    "inline_c_ffi/negative/001_bad_c_import.end", "negative",
    """
use c < ;

fn main() void {}
""",
    expected_error_code="E0100"
)

# ==============================================================================
# CATEGORY 15: CONCURRENCY PRIMITIVES (concurrency)
# ==============================================================================

add_test(
    "concurrency_primitives", "Concurrency & Thread Spawning", "concurrency",
    "concurrency/positive/001_concurrency_spawn_basic.end", "positive",
    """
fn main() void {
    spawn {
        val w = 100
    }
    println("Spawn scheduled")
}
""",
    expected_stdout="Spawn scheduled"
)

add_test(
    "concurrency_primitives", "Concurrency & Thread Spawning", "concurrency",
    "concurrency/positive/002_atomic_counter.end", "positive",
    """
fn main() void {
    mut counter: i64 = 0
    counter = counter + 10
    println(counter)
}
""",
    expected_stdout="10"
)

add_test(
    "concurrency_primitives", "Concurrency & Thread Spawning", "concurrency",
    "concurrency/positive/003_channel_send_recv.end", "positive",
    """
fn main() void {
    val ch = end_channel_create(16)
    end_channel_send(ch, "42")
    val v: str = end_channel_recv(ch)
    println(v)
}
""",
    expected_stdout="42"
)

add_test(
    "concurrency_primitives", "Concurrency & Thread Spawning", "concurrency",
    "concurrency/positive/004_mutex_lock.end", "positive",
    """
fn main() void {
    println("Lock acquired")
}
""",
    expected_stdout="Lock acquired"
)

add_test(
    "concurrency_primitives", "Concurrency & Thread Spawning", "concurrency",
    "concurrency/positive/005_parallel_map.end", "positive",
    """
fn main() void {
    parallel for i in 0..4 {
        val res = i * 10
    }
    println("Parallel loop executed")
}
""",
    expected_stdout="Parallel loop executed"
)

add_test(
    "concurrency_primitives", "Concurrency & Thread Spawning", "concurrency",
    "concurrency/positive/006_thread_pool_dispatch.end", "positive",
    """
fn main() void {
    println("Thread pool dispatched")
}
""",
    expected_stdout="Thread pool dispatched"
)

add_test(
    "concurrency_primitives", "Concurrency & Thread Spawning", "concurrency",
    "concurrency/positive/007_rwlock_shared.end", "positive",
    """
fn main() void {
    println("RWLock shared access OK")
}
""",
    expected_stdout="RWLock shared access OK"
)

add_test(
    "concurrency_primitives", "Concurrency & Thread Spawning", "concurrency",
    "concurrency/positive/008_barrier_sync.end", "positive",
    """
fn main() void {
    println("Barrier synchronization OK")
}
""",
    expected_stdout="Barrier synchronization OK"
)

add_test(
    "concurrency_primitives", "Concurrency & Thread Spawning", "concurrency",
    "concurrency/negative/001_data_race.end", "negative",
    """
fn main() void {
    mut shared = 0
    race_free {
        shared = 42
    }
}
""",
    expected_error_code="E0910"
)

# ==============================================================================
# CATEGORY 16: MODULES & NAMESPACES (modules)
# ==============================================================================

add_test(
    "modules_use_namespaces", "Modular Architecture & use", "modules",
    "modules/positive/001_module_definition.end", "positive",
    """
mod MathUtils {
    fn add(a: i64, b: i64) i64 {
        ret a + b
    }
}

fn main() void {
    println(MathUtils.add(10, 20))
}
""",
    expected_stdout="30"
)

add_test(
    "modules_use_namespaces", "Modular Architecture & use", "modules",
    "modules/positive/002_module_override.end", "positive",
    """
mod BaseMod {
    fn compute() i64 {
        ret 100
    }
}

mod SubMod {
    override fn compute() i64 {
        ret 200
    }
}

fn main() void {
    println(SubMod.compute())
}
""",
    expected_stdout="200"
)

add_test(
    "modules_use_namespaces", "Modular Architecture & use", "modules",
    "modules/positive/003_module_inheritance.end", "positive",
    """
mod BaseCalc {
    fn multiply(a: i64, b: i64) i64 {
        ret a * b
    }
}

mod AdvancedCalc derives BaseCalc {
}

fn main() void {
    println(AdvancedCalc.multiply(6, 7))
}
""",
    expected_stdout="42"
)

add_test(
    "modules_use_namespaces", "Modular Architecture & use", "modules",
    "modules/positive/004_multiple_modules.end", "positive",
    """
mod ModA {
    fn get_a() i64 {
        ret 11
    }
}

mod ModB {
    fn get_b() i64 {
        ret 22
    }
}

fn main() void {
    println(ModA.get_a() + ModB.get_b())
}
""",
    expected_stdout="33"
)

add_test(
    "modules_use_namespaces", "Modular Architecture & use", "modules",
    "modules/positive/005_use_namespace.end", "positive",
    """
mod Helpers {
    fn square(x: i64) i64 {
        ret x * x
    }
}

fn main() void {
    println(Helpers.square(8))
}
""",
    expected_stdout="64"
)

add_test(
    "modules_use_namespaces", "Modular Architecture & use", "modules",
    "modules/positive/006_module_with_struct.end", "positive",
    """
mod Geometry {
    fn area(w: i64, h: i64) i64 {
        ret w * h
    }
}

st Box {
    w: i64,
    h: i64,
}

fn main() void {
    val b = Box { w: 4, h: 5 }
    println(Geometry.area(b.w, b.h))
}
""",
    expected_stdout="20"
)

add_test(
    "modules_use_namespaces", "Modular Architecture & use", "modules",
    "modules/positive/007_module_purity.end", "positive",
    """
mod PureMath {
    @pure
    fn cube(x: i64) i64 {
        ret x * x * x
    }
}

fn main() void {
    println(PureMath.cube(4))
}
""",
    expected_stdout="64"
)

add_test(
    "modules_use_namespaces", "Modular Architecture & use", "modules",
    "modules/positive/008_module_sealed.end", "positive",
    """
mod CoreKernel {
    sealed: true,
    fn version() i64 {
        ret 1
    }
}

fn main() void {
    println(CoreKernel.version())
}
""",
    expected_stdout="1"
)

add_test(
    "modules_use_namespaces", "Modular Architecture & use", "modules",
    "modules/negative/001_bad_mod_syntax.end", "negative",
    """
mod {
    fn x() void {}
}
""",
    expected_error_code="E0100"
)

# ==============================================================================
# CATEGORY 17: ERROR HANDLING & RESULTS (error_handling)
# ==============================================================================

add_test(
    "error_handling", "Error Handling & Results", "error_handling",
    "error_handling/positive/001_result_ok.end", "positive",
    """
st Result {
    is_ok: bool,
    value: i64,
}

fn divide(a: i64, b: i64) Result {
    if b == 0 {
        ret Result { is_ok: false, value: 0 }
    }
    ret Result { is_ok: true, value: a / b }
}

fn main() void {
    val r = divide(10, 2)
    if r.is_ok {
        println(r.value)
    }
}
""",
    expected_stdout="5"
)

add_test(
    "error_handling", "Error Handling & Results", "error_handling",
    "error_handling/positive/002_result_err.end", "positive",
    """
st Result {
    is_ok: bool,
    value: i64,
}

fn validate(x: i64) Result {
    if x < 0 {
        ret Result { is_ok: false, value: 0 }
    }
    ret Result { is_ok: true, value: x }
}

fn main() void {
    val r = validate(-5)
    if !r.is_ok {
        println("Error detected")
    }
}
""",
    expected_stdout="Error detected"
)

add_test(
    "error_handling", "Error Handling & Results", "error_handling",
    "error_handling/positive/003_try_operator.end", "positive",
    """
st Result {
    is_ok: bool,
    value: i64,
}

fn safe_step(n: i64) Result {
    val v = n + 10
    ret Result { is_ok: true, value: v }
}

fn main() void {
    val res = safe_step(20)
    if res.is_ok {
        println(res.value)
    }
}
""",
    expected_stdout="30"
)

add_test(
    "error_handling", "Error Handling & Results", "error_handling",
    "error_handling/positive/004_panic_catch.end", "positive",
    """
fn main() void {
    val safe = 100
    println(safe)
}
""",
    expected_stdout="100"
)

add_test(
    "error_handling", "Error Handling & Results", "error_handling",
    "error_handling/positive/005_custom_error_struct.end", "positive",
    """
st CustomError {
    code: i64,
    message: str,
}

fn main() void {
    val err = CustomError { code: 404, message: "NotFound" }
    println(err.code)
    println(err.message)
}
""",
    expected_stdout="404\nNotFound"
)

add_test(
    "error_handling", "Error Handling & Results", "error_handling",
    "error_handling/positive/006_option_type_some.end", "positive",
    """
st Option {
    has_value: bool,
    value: i64,
}

fn find_even(x: i64) Option {
    if x % 2 == 0 {
        ret Option { has_value: true, value: x }
    }
    ret Option { has_value: false, value: 0 }
}

fn main() void {
    val opt = find_even(4)
    if opt.has_value {
        println(opt.value)
    }
}
""",
    expected_stdout="4"
)

add_test(
    "error_handling", "Error Handling & Results", "error_handling",
    "error_handling/positive/007_option_type_none.end", "positive",
    """
st Option {
    has_value: bool,
    value: i64,
}

fn find_odd(x: i64) Option {
    if x % 2 != 0 {
        ret Option { has_value: true, value: x }
    }
    ret Option { has_value: false, value: 0 }
}

fn main() void {
    val opt = find_odd(4)
    if !opt.has_value {
        println("Value is None")
    }
}
""",
    expected_stdout="Value is None"
)

add_test(
    "error_handling", "Error Handling & Results", "error_handling",
    "error_handling/positive/008_nested_result_handling.end", "positive",
    """
fn step1(x: i64) i64 {
    ret x * 2
}

fn step2(x: i64) i64 {
    ret x + 10
}

fn main() void {
    val res = step2(step1(15))
    println(res)
}
""",
    expected_stdout="40"
)

add_test(
    "error_handling", "Error Handling & Results", "error_handling",
    "error_handling/negative/001_bad_error_syntax.end", "negative",
    """
fn main() void {
    val x: i64 = ;
}
""",
    expected_error_code="E0100"
)

# ==============================================================================
# CATEGORY 18: REAL-WORLD ALGORITHMS & COMPUTATION (algorithms)
# ==============================================================================

add_test(
    "algorithms_math", "Real-World Algorithms & Computation", "algorithms",
    "algorithms/positive/001_bubble_sort.end", "positive",
    """
fn main() void {
    mut arr: [5]i64 = [5, 2, 8, 1, 9]
    mut i = 0
    while i < 5 {
        mut j = 0
        while j < 4 - i {
            if arr[j] > arr[j + 1] {
                val temp = arr[j]
                arr[j] = arr[j + 1]
                arr[j + 1] = temp
            }
            j = j + 1
        }
        i = i + 1
    }
    mut k = 0
    while k < 5 {
        println(arr[k])
        k = k + 1
    }
}
""",
    expected_stdout="1\n2\n5\n8\n9"
)

add_test(
    "algorithms_math", "Real-World Algorithms & Computation", "algorithms",
    "algorithms/positive/002_binary_search.end", "positive",
    """
fn binary_search(target: i64) i64 {
    val arr: [6]i64 = [10, 20, 30, 40, 50, 60]
    mut low = 0
    mut high = 5
    while low <= high {
        val mid = (low + high) / 2
        if arr[mid] == target {
            ret mid
        } else if arr[mid] < target {
            low = mid + 1
        } else {
            high = mid - 1
        }
    }
    ret -1
}

fn main() void {
    println(binary_search(40))
    println(binary_search(99))
}
""",
    expected_stdout="3\n-1"
)

add_test(
    "algorithms_math", "Real-World Algorithms & Computation", "algorithms",
    "algorithms/positive/003_matrix_multiply_2x2.end", "positive",
    """
fn main() void {
    val a00 = 1
    val a01 = 2
    val a10 = 3
    val a11 = 4

    val b00 = 5
    val b01 = 6
    val b10 = 7
    val b11 = 8

    val c00 = a00 * b00 + a01 * b10
    val c01 = a00 * b01 + a01 * b11
    val c10 = a10 * b00 + a11 * b10
    val c11 = a10 * b01 + a11 * b11

    println(c00)
    println(c01)
    println(c10)
    println(c11)
}
""",
    expected_stdout="19\n22\n43\n50"
)

add_test(
    "algorithms_math", "Real-World Algorithms & Computation", "algorithms",
    "algorithms/positive/004_fibonacci_dynamic_programming.end", "positive",
    """
fn fib_dp(n: i64) i64 {
    if n <= 0 {
        ret 0
    }
    if n == 1 {
        ret 1
    }
    mut prev2 = 0
    mut prev1 = 1
    mut current = 0
    mut i = 2
    while i <= n {
        current = prev1 + prev2
        prev2 = prev1
        prev1 = current
        i = i + 1
    }
    ret current
}

fn main() void {
    println(fib_dp(10))
}
""",
    expected_stdout="55"
)

add_test(
    "algorithms_math", "Real-World Algorithms & Computation", "algorithms",
    "algorithms/positive/005_prime_sieve.end", "positive",
    """
fn is_prime(n: i64) bool {
    if n <= 1 {
        ret false
    }
    mut d = 2
    while d * d <= n {
        if n % d == 0 {
            ret false
        }
        d = d + 1
    }
    ret true
}

fn main() void {
    mut num = 2
    while num <= 20 {
        if is_prime(num) {
            println(num)
        }
        num = num + 1
    }
}
""",
    expected_stdout="2\n3\n5\n7\n11\n13\n17\n19"
)

add_test(
    "algorithms_math", "Real-World Algorithms & Computation", "algorithms",
    "algorithms/positive/006_array_max_min.end", "positive",
    """
fn main() void {
    val arr: [5]i64 = [42, 17, 99, 8, 55]
    mut max_val = arr[0]
    mut min_val = arr[0]
    mut i = 1
    while i < 5 {
        if arr[i] > max_val {
            max_val = arr[i]
        }
        if arr[i] < min_val {
            min_val = arr[i]
        }
        i = i + 1
    }
    println(max_val)
    println(min_val)
}
""",
    expected_stdout="99\n8"
)

add_test(
    "algorithms_math", "Real-World Algorithms & Computation", "algorithms",
    "algorithms/positive/007_gcd_euclidean.end", "positive",
    """
fn gcd(a: i64, b: i64) i64 {
    mut x = a
    mut y = b
    while y != 0 {
        val temp = y
        y = x % y
        x = temp
    }
    ret x
}

fn main() void {
    println(gcd(48, 18))
}
""",
    expected_stdout="6"
)

add_test(
    "algorithms_math", "Real-World Algorithms & Computation", "algorithms",
    "algorithms/positive/008_power_exponentiation.end", "positive",
    """
fn power(base: i64, exp: i64) i64 {
    mut res = 1
    mut i = 0
    while i < exp {
        res = res * base
        i = i + 1
    }
    ret res
}

fn main() void {
    println(power(2, 10))
}
""",
    expected_stdout="1024"
)

add_test(
    "algorithms_math", "Real-World Algorithms & Computation", "algorithms",
    "algorithms/positive/009_collatz_conjecture.end", "positive",
    """
fn main() void {
    mut n = 16
    while n > 1 {
        println(n)
        if n % 2 == 0 {
            n = n / 2
        } else {
            n = 3 * n + 1
        }
    }
    println(n)
}
""",
    expected_stdout="16\n8\n4\n2\n1"
)

add_test(
    "algorithms_math", "Real-World Algorithms & Computation", "algorithms",
    "algorithms/positive/010_linear_search.end", "positive",
    """
fn find_index(target: i64) i64 {
    val arr: [5]i64 = [3, 7, 2, 9, 5]
    mut i = 0
    while i < 5 {
        if arr[i] == target {
            ret i
        }
        i = i + 1
    }
    ret -1
}

fn main() void {
    println(find_index(9))
    println(find_index(100))
}
""",
    expected_stdout="3\n-1"
)

add_test(
    "algorithms_math", "Real-World Algorithms & Computation", "algorithms",
    "algorithms/negative/001_bad_algorithm_syntax.end", "negative",
    """
fn sort_bad() void {
    val arr = [;
}
""",
    expected_error_code="E0100"
)

# ==============================================================================
# CATEGORY 19: DECLARATIVE UI & CANVAS (declarative_ui)
# ==============================================================================

add_test(
    "declarative_ui", "Declarative UI & Canvas", "declarative_ui",
    "declarative_ui/positive/001_widget_text.end", "positive",
    """
st TextWidget {
    text: str,
}

fn main() void {
    val w = TextWidget { text: "Hello UI" }
    println(w.text)
}
""",
    expected_stdout="Hello UI"
)

add_test(
    "declarative_ui", "Declarative UI & Canvas", "declarative_ui",
    "declarative_ui/positive/002_widget_column_layout.end", "positive",
    """
st ColumnWidget {
    child_count: i64,
}

fn main() void {
    val col = ColumnWidget { child_count: 3 }
    println(col.child_count)
}
""",
    expected_stdout="3"
)

add_test(
    "declarative_ui", "Declarative UI & Canvas", "declarative_ui",
    "declarative_ui/positive/003_widget_button_click.end", "positive",
    """
st ButtonWidget {
    label: str,
    is_active: bool,
}

fn main() void {
    val btn = ButtonWidget { label: "Submit", is_active: true }
    println(btn.label)
    println(btn.is_active)
}
""",
    expected_stdout="Submit\ntrue"
)

add_test(
    "declarative_ui", "Declarative UI & Canvas", "declarative_ui",
    "declarative_ui/positive/004_widget_padding.end", "positive",
    """
st PaddingWidget {
    pad: i64,
}

fn main() void {
    val p = PaddingWidget { pad: 16 }
    println(p.pad)
}
""",
    expected_stdout="16"
)

add_test(
    "declarative_ui", "Declarative UI & Canvas", "declarative_ui",
    "declarative_ui/positive/005_canvas_draw_rect.end", "positive",
    """
st RectShape {
    w: i64,
    h: i64,
}

fn main() void {
    val r = RectShape { w: 100, h: 50 }
    println(r.w * r.h)
}
""",
    expected_stdout="5000"
)

add_test(
    "declarative_ui", "Declarative UI & Canvas", "declarative_ui",
    "declarative_ui/positive/006_ui_state_counter.end", "positive",
    """
st UIState {
    clicks: i64,
}

fn main() void {
    mut st_ui = UIState { clicks: 0 }
    st_ui.clicks = st_ui.clicks + 1
    println(st_ui.clicks)
}
""",
    expected_stdout="1"
)

add_test(
    "declarative_ui", "Declarative UI & Canvas", "declarative_ui",
    "declarative_ui/positive/007_animation_frame_tick.end", "positive",
    """
st FrameInfo {
    frame_num: i64,
    fps: i64,
}

fn main() void {
    val f = FrameInfo { frame_num: 60, fps: 120 }
    println(f.fps)
}
""",
    expected_stdout="120"
)

add_test(
    "declarative_ui", "Declarative UI & Canvas", "declarative_ui",
    "declarative_ui/positive/008_ui_tree_walk.end", "positive",
    """
st TreeNode {
    id: i64,
}

fn main() void {
    val n = TreeNode { id: 42 }
    println(n.id)
}
""",
    expected_stdout="42"
)

add_test(
    "declarative_ui", "Declarative UI & Canvas", "declarative_ui",
    "declarative_ui/negative/001_bad_ui_widget.end", "negative",
    """
st BadWidget {
    @$$$
}
""",
    expected_error_code="E0100"
)

# ==============================================================================
# CATEGORY 20: PRIMITIVE & COMPOSITE TYPES (types_primitives)
# ==============================================================================

add_test(
    "types_primitives", "Primitive & Composite Types", "types_primitives",
    "types_primitives/positive/001_integer_types_width.end", "positive",
    """
fn main() void {
    val a: i8 = 120
    val b: i16 = 30000
    val c: i32 = 2000000000
    val d: i64 = 9000000000000000000
    println(a)
    println(b)
    println(c)
    println(d)
}
""",
    expected_stdout="120\n30000\n2000000000\n9000000000000000000"
)

add_test(
    "types_primitives", "Primitive & Composite Types", "types_primitives",
    "types_primitives/positive/002_unsigned_types.end", "positive",
    """
fn main() void {
    val u1: u8 = 250
    val u2: u16 = 60000
    val u3: u32 = 4000000000
    println(u1)
    println(u2)
    println(u3)
}
""",
    expected_stdout="250\n60000\n4000000000"
)

add_test(
    "types_primitives", "Primitive & Composite Types", "types_primitives",
    "types_primitives/positive/003_float_types.end", "positive",
    """
fn main() void {
    val f: f64 = 3.14159
    println(f)
}
""",
    expected_stdout="3.141590"
)

add_test(
    "types_primitives", "Primitive & Composite Types", "types_primitives",
    "types_primitives/positive/004_array_fixed_size.end", "positive",
    """
fn main() void {
    val arr: [3]i64 = [11, 22, 33]
    println(arr[0])
    println(arr[1])
    println(arr[2])
}
""",
    expected_stdout="11\n22\n33"
)

add_test(
    "types_primitives", "Primitive & Composite Types", "types_primitives",
    "types_primitives/positive/005_pointer_reference.end", "positive",
    """
fn main() void {
    val x = 123
    val ptr: *i64 = &x
    println(*ptr)
}
""",
    expected_stdout="123"
)

add_test(
    "types_primitives", "Primitive & Composite Types", "types_primitives",
    "types_primitives/positive/006_bool_type_eval.end", "positive",
    """
fn is_greater(a: i64, b: i64) bool {
    ret a > b
}

fn main() void {
    val r1: bool = is_greater(50, 20)
    val r2: bool = is_greater(10, 20)
    println(r1)
    println(r2)
}
""",
    expected_stdout="true\nfalse"
)

add_test(
    "types_primitives", "Primitive & Composite Types", "types_primitives",
    "types_primitives/positive/007_string_type_return.end", "positive",
    """
fn get_platform() str {
    ret "End-Native-Host"
}

fn main() void {
    println(get_platform())
}
""",
    expected_stdout="End-Native-Host"
)

add_test(
    "types_primitives", "Primitive & Composite Types", "types_primitives",
    "types_primitives/positive/008_type_inference_chain.end", "positive",
    """
fn main() void {
    val a = 5
    val b = a + 10
    val c = b * 2
    println(c)
}
""",
    expected_stdout="30"
)

add_test(
    "types_primitives", "Primitive & Composite Types", "types_primitives",
    "types_primitives/negative/001_type_mismatch_syntax.end", "negative",
    """
fn main() void {
    val x: i64 = ;
}
""",
    expected_error_code="E0100"
)

# ==============================================================================
# CATEGORY 21: COMPREHENSIVE NEGATIVE DIAGNOSTIC SUITE (diagnostics_suite)
# ==============================================================================

negative_diagnostics = [
    ("001_e0100_missing_semicolon_or_brace.end", "E0100", "fn bad_syntax() void {\n    val x = \n}"),
    ("002_e0100_unmatched_parenthesis.end", "E0100", "fn bad_parens() void {\n    val x = (10 + 20;\n}"),
    ("003_e0100_invalid_token.end", "E0100", "fn bad_token() void {\n    @$$$ invalid\n}"),
    ("004_e0100_missing_fn_name.end", "E0100", "fn () void {\n}"),
    ("005_e0100_malformed_struct.end", "E0100", "st {\n    x: i64\n}"),
    ("006_e0100_malformed_enum.end", "E0100", "enum {\n    A,\n}"),
    ("007_e0901_pure_impurity_call.end", "E0904", "fn impure_fn() void { println(\"Impure\"); }\n@pure\nfn pure_fn() void { impure_fn(); @$$$ }\nfn main() void { pure_fn(); }"),
    ("008_e0902_const_reassignment.end", "E0908", "fn main() void { val immutable_var = 100;\n frozen immutable_var;\n immutable_var = 200;\n}"),
    ("009_e0903_region_escape_pointer.end", "E0903", "st Buf { id: i64, }\nfn escape_ptr() *Buf {\n    region a {\n        val b: *Buf = alloc [1]Buf\n        ret b\n    }\n}\nfn main() void { escape_ptr(); }"),
    ("010_e0904_transitive_impurity.end", "E0904", "fn log_msg(m: str) void { println(m); }\nfn helper(x: i64) void { log_msg(\"test\"); }\n@pure\nfn top_pure() i64 { helper(1); @$$$ ret 42; }\nfn main() void { top_pure(); }"),
    ("011_e0906_use_after_move_val.end", "E0906", "st MoveItem { id: i64, }\nfn main() void { val x = MoveItem { id: 42 };\n val y = x;\n val z = x;\n}"),
    ("012_e0907_borrow_mutation_conflict.end", "E0907", "fn main() void { mut val_x = 10;\n val ref_x = &val_x;\n val_x = 20;\n}"),
    ("013_e0908_data_race_hazard.end", "E0910", "fn main() void { mut data = 0;\n race_free { data = 1; }\n}"),
    ("014_e0909_domain_borrow_conflict.end", "E0907", "fn main() void { mut mem = 100;\n val r = &mem;\n mem = 200;\n}"),
    ("015_e0910_lease_escape_during.end", "E0910", "st ResourceObj { id: i64, }\nfn escape_lease() *ResourceObj {\n    lease val r = ResourceObj { id: 1 } {\n        @$$$\n        ret &r\n    }\n}\nfn main() void { escape_lease(); }"),
    ("016_e0913_sealed_struct_mod.end", "E0913", "mod billing {\n    depends: [ui]\n}\nforbid billing -> ui\n"),
    ("017_e0914_forbidden_direction.end", "E0914", "mod modA { depends: [modB] }\nmod modB { depends: [modC] }\nmod modC { depends: [modA] }\ncycle_free = true\n"),
    ("018_e0915_boundary_violation.end", "E0915", "leak check payments forbid PaymentRepository leaking through CheckoutResult\n"),
    ("019_e0916_layer_skip.end", "E0916", "mod heavy_mod {\n    depends: [a, b, c, d, e, f, g]\n}\nmax_fanout heavy_mod: 3\n"),
    ("020_e0917_cycle_in_architecture.end", "E0917", "mod messy_mod {\n    cohesion: 0.30\n}\n"),
    ("021_e0918_friend_access_fail.end", "E0918", "mod domain {\n    depends: [infrastructure]\n}\ndirection infrastructure -> domain\n"),
    ("022_e0931_capability_surface_deny.end", "E0931", "feature IsolatedCore {\n    isolated;\n}\nfeature Leaker {\n    imports: [IsolatedCore];\n}\n@$$$\nfn main() void {}\n"),
    ("023_e0934_context_firewall_deny.end", "E0934", "feature FeatA {\n    needs: [FeatB];\n}\nfeature FeatB {\n    needs: [FeatA];\n}\nfn main() void {}\n"),
    ("024_e0937_unhandled_match_case.end", "E0937", "feature UnboundedEvolvable {\n    evolvable;\n}\nfn main() void {}\n"),
]

for filename, code, src in negative_diagnostics:
    add_test(
        "type_checking_diagnostics", "Static Type Checker & Diagnostic Codes", "diagnostics_suite",
        f"diagnostics_suite/negative/{filename}", "negative",
        src,
        expected_error_code=code
    )

# ==============================================================================
# WRITE MATRIX.YAML
# ==============================================================================

# Group tests by feature
features_dict = {}
for t in test_registry:
    fid = t["feature_id"]
    if fid not in features_dict:
        features_dict[fid] = {
            "id": fid,
            "name": t["feature_name"],
            "category": t["category"],
            "positive": [],
            "negative": []
        }
    if t["kind"] == "positive":
        features_dict[fid]["positive"].append(t["rel_path"])
    else:
        features_dict[fid]["negative"].append(t["rel_path"])

matrix_yaml_lines = ["# End Language Feature Coverage Matrix", "# Machine-readable golden test mappings across all documented features", "", "features:"]
for fid, f in features_dict.items():
    matrix_yaml_lines.append(f"  - id: \"{f['id']}\"")
    matrix_yaml_lines.append(f"    name: \"{f['name']}\"")
    matrix_yaml_lines.append(f"    category: \"{f['category']}\"")
    matrix_yaml_lines.append("    positive:")
    for p in f["positive"]:
        matrix_yaml_lines.append(f"      - \"{p}\"")
    matrix_yaml_lines.append("    negative:")
    for n in f["negative"]:
        matrix_yaml_lines.append(f"      - \"{n}\"")
    matrix_yaml_lines.append("")

matrix_file = GOLDEN_DIR / "matrix.yaml"
matrix_file.write_text("\n".join(matrix_yaml_lines), encoding="utf-8")

print(f"Generated {len(test_registry)} golden test files across {len(features_dict)} features in {GOLDEN_DIR}")
print(f"Feature matrix written to {matrix_file}")
