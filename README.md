# GR8

## Features

### Disabling Main Function

This project includes a feature flag to disable running the main function. By default, the main function is enabled.

To build without the main function:

```bash
cargo build --no-default-features
```

To run with the main function (default):

```bash
cargo run
```

Or explicitly:

```bash
cargo run --features run-main
```

When the main function is disabled, a simple message will be printed instead of running the graphical application.
