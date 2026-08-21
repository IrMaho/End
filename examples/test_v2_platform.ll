; ModuleID = 'main'
target triple = "x86_64-pc-windows-msvc"
source_filename = "main.end"

declare i32 @printf(i8*, ...)

define void @test_pillar1_generics_and_adts() {
entry:
  %ok_res = add i64 0, 0
  %t0 = call i64 @result_is_ok(i64 %ok_res)
  %is_ok = add i64 %t0, 0
  %some_opt = add i64 0, 0
  %t1 = call i64 @option_is_some(i64 %some_opt)
  %is_some = add i64 %t1, 0
  %t2 = call i64 @list_create(i64 8)
  %initial_list = add i64 %t2, 0
  %t3 = call i64 @list_push(i64 %initial_list, i64 999)
  %updated_list = add i64 %t3, 0
  %t4 = call i64 @map_create(i64 16)
  %initial_map = add i64 %t4, 0
  %t5 = call i64 @map_insert(i64 %initial_map, i64 0, i64 0)
  %updated_map = add i64 %t5, 0
  %t6 = call i64 @ring_create(i64 32)
  %initial_ring = add i64 %t6, 0
  %t7 = call i64 @ring_push(i64 %initial_ring, i64 12345)
  %updated_ring = add i64 %t7, 0
  ret void
}

define void @test_pillar2_c10m_async_and_fibers() {
entry:
  %t8 = call i64 @async_driver_create_native()
  %loop_inst = add i64 %t8, 0
  %t9 = call i64 @async_driver_register_fd(i64 %loop_inst, i64 1024)
  %registered = add i64 %t9, 0
  %t10 = call i64 @async_driver_poll(i64 %registered, i64 10)
  %polled = add i64 %t10, 0
  %t11 = call i64 @fiber_scheduler_create()
  %sched = add i64 %t11, 0
  %t12 = call i64 @fiber_spawn(i64 %sched, i64 1001)
  %spawn_res = add i64 %t12, 0
  %t13 = call i64 @fiber_yield_step(i64 0, i64 0)
  %yield_res = add i64 %t13, 0
  ret void
}

define void @test_pillar3_tls13_http2_and_acme() {
entry:
  %t14 = call i64 @tls13_handshake_server(i64 0)
  %tls_session = add i64 %t14, 0
  %t15 = call i64 @tls13_encrypt_record(i64 %tls_session, i64 1024)
  %encrypted_session = add i64 %t15, 0
  %t16 = call i64 @http2_mux_create(i64 1000)
  %mux = add i64 %t16, 0
  %t17 = call i64 @http2_mux_open_stream(i64 %mux, i64 1)
  %mux_res = add i64 %t17, 0
  %t18 = call i64 @acme_order_certificate(i64 0)
  %cert = add i64 %t18, 0
  ret void
}

define void @test_pillar5_global_registry_cdn() {
entry:
  %t19 = call i64 @registry_client_create()
  %client = add i64 %t19, 0
  %t20 = call i64 @registry_query_package(i64 %client, i64 0)
  %query_res = add i64 %t20, 0
  ret void
}

define void @main() {
entry:
  ret void
}

