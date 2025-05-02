# "Voltchat" Example

This example demonstrates a simple chat app (client and server, command-line) which supports as many clients as you have ports available (a lot).

This is built on Voltlane ([voltlane.net](https://voltlane.net)), a free & open source connection server written in Rust (with bindings for other languages).
Voltlane makes it possible for this server to be *so* simple, and for the client to be so lean, without losing out on the benefits of using TCP.

## How to use

1. Run `cargo run --bin server`
2. Clone the Voltlane connection server from https://github.com/voltlane/connserver and run it with `cargo run --bin connserver`
3. Run `cargo run --bin client` some $N$ amount of times and start chatting! :)

![image](https://github.com/user-attachments/assets/7340e2f5-d5f2-49e6-8cdd-c2bf261b0935)
