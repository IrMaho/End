pub mod concurrency_crypto;
pub mod core;
pub mod desktop_and_memory;
pub mod network_sockets;
pub mod tls_tensor_canvas;

pub fn emit_all_runtime_headers(out: &mut String) {
    core::emit_core_runtime(out);
    desktop_and_memory::emit_desktop_and_memory_runtime(out);
    network_sockets::emit_network_sockets_runtime(out);
    concurrency_crypto::emit_concurrency_crypto_runtime(out);
    tls_tensor_canvas::emit_tls_tensor_canvas_runtime(out);
}
