# NAME

Email Alerting Micro Service Component

# DESCRIPTION

This service provides Email Sending as _HTTP API_.

# REQUIREMENTS

To rebuild this web site the **Minimum Rust Compiler Version** is _Rust_ `1.49`.
The site uses the libraries `Actix`, `Serde` and `json-rust`.
The _Actix_ Web Server requires the `Tokio` library.
The Server Responses are provided as `JSON` documents.

# INSTALLATION

- cargo

The `cargo` Command will install the dependencies on local user level as they
are found in the `Cargo.toml` file.

# EXECUTION

- `cargo run`

The Site can be launched using the `cargo run` Command.
To launch the Site call the `cargo run` Command within the project directory:

            cargo run

# IMPLEMENTATION

- Actor Model

To not block the server main thread too long and to enable asynchronous request processing
the `Actor` trait of _Actix_ and `Future`s are used.

