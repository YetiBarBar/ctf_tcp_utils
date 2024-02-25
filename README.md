# ctf_tcp_helper
Wrapper library for TCP based CTF

# Description
Wrapper to manage TCP connections in CTF challs.

# Usage
```
fn main() -> anyhow::Result<()> {
    let url = "challenge.com";
    let port = 4242;
    let mut tcp_handle = ctf_tcp_utils::TcpHandler::new(&url, port)?;
    let input = tcp_handle.read_to_string();
    println!("{input}");
    let answer = process(&input);
    println!("{answer}");
    tcp_handle.write_answer(&answer);
    let result = tcp_handle.read_to_string();
    println!("{result}");
    Ok(())
}
```

If the CTF requires to repeat the same business logic in a loop, you can try:
```
fn main() {
    let result = ctf_tcp_utils::run_function_loop("localhost", 4000, alenvers_business).unwrap();
    println!("{result}");
}
```