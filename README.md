# RobotS

[![pre-commit.ci status](https://results.pre-commit.ci/badge/github/nim65s/RobotS/main.svg)](https://results.pre-commit.ci/latest/github/nim65s/RobotS/main)

Companion project to the [Rust RoverS](https://homepages.laas.fr/gsaurel/talks/rust-rovers.pdf) talk, which present
Rust to Roboticists, including the [rust-embedded](https://github.com/rust-embedded) community with an
[embassy](https://github.com/embassy-rs/embassy) example, and the rust web community with a
[leptos](https://github.com/leptos-rs/leptos) example.

This mostly show how we can design a message system with Rust enums, and serialize them to exchange messages between a
webbrowser (eg. in HTTP) and a microcontroller (eg. in UART) through a simple server.

It works on a ESP32-C3-DevKitC-02v1.1 and a bluepill.
