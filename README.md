# RobotS

Companion project to the [Rust RoverS](https://homepages.laas.fr/gsaurel/talks/rust-rovers.pdf) talk, which present
Rust to Roboticists, including the [rust-embedded](https://github.com/rust-embedded) community with an
[embassy](https://github.com/embassy-rs/embassy) example, and the rust web community with a
[leptos](https://github.com/leptos-rs/leptos) example.

This mostly show how we can design a message system with Rust enums, and serialize them to exchange messages between a
webbrowser (eg. in HTTP) and a microcontroller (eg. in UART) through a simple server.
