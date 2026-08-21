; ModuleID = 'main'
target triple = "x86_64-pc-windows-msvc"
source_filename = "main.end"

declare i32 @printf(i8*, ...)
declare i8* @malloc(i64)
declare void @free(i8*)

%struct.Point = type { i64, i64 }

define %struct.Point @make_point(i64 %arg_x, i64 %arg_y) {
entry:
  %x = alloca i64
  store i64 %arg_x, i64* %x
  %y = alloca i64
  store i64 %arg_y, i64* %y
  %p = alloca i64
  store i64 0, i64* %p
  %t0 = load i64, i64* %p
  ret i64 %t0
}

define void @test_valid_region() {
entry:
  %pt = alloca i64
  %t1 = call i64 @make_point(i64 10, i64 20)
  store i64 %t1, i64* %pt
  %t2 = call i64 @assert_eq_i64(i64 0, i64 10)
  %t3 = call i64 @assert_eq_i64(i64 0, i64 20)
  ret void
}

