# "Voltchat" Example

This example demonstrates a simple chat app (client and server, command-line) which supports as many clients as you have ports available (a lot).

This is built on Voltlane ([voltlane.net](https://voltlane.net)), a free & open source connection server written in Rust (with bindings for other languages).
Voltlane makes it possible for this server to be *so* simple, and for the client to be so lean, without losing out on the benefits of using TCP.
