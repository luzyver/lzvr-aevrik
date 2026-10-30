use std::{io::Write, net::TcpListener, time::Duration};
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("exit") {
        eprintln!("intentional failure");
        std::process::exit(7);
    }
    if args.first().map(String::as_str) == Some("echo") {
        println!("{}", args.get(1).map(String::as_str).unwrap_or(""));
        return;
    }
    if args.first().map(String::as_str) == Some("tree") {
        let mut child = std::process::Command::new(std::env::current_exe().expect("exe"))
            .arg(args.get(1).expect("child port"))
            .spawn()
            .expect("child");
        // Keep the parent alive. The engine's Windows Job must own both processes.
        child.wait().expect("wait for child");
        return;
    }
    let port: u16 = args.first().expect("port").parse().expect("valid port");
    let _listener = TcpListener::bind(("127.0.0.1", port)).expect("listen");
    println!("fixture ready");
    std::io::stdout().flush().expect("flush");
    loop {
        std::thread::sleep(Duration::from_millis(50));
    }
}
