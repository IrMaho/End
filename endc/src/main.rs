fn main() {
    let builder = std::thread::Builder::new().stack_size(16 * 1024 * 1024);
    let handler = builder
        .spawn(|| {
            endc::run_app();
        })
        .unwrap();
    if let Err(e) = handler.join() {
        eprintln!("Fatal error in End compiler execution: {:?}", e);
        std::process::exit(1);
    }
}
