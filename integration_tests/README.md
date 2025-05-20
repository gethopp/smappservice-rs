# Integration Tests for SMAppService-RS

This directory contains integration tests for the smappservice-rs library. Each subdirectory is a separate test crate that tests a different service type:

- `test_mainapp` - Tests the main application service type
- `test_agent` - Tests the launch agent service type
- `test_daemon` - Tests the launch daemon service type
- `test_loginitem` - Tests the login item service type
- `test-app` - The test application that is used by all test crates

## Requirements

- `cargo-bundle` (install with `cargo install cargo-bundle`)

## Running Tests

```bash
cargo run --bin test_mainapp
```

## How It Works

Each test uses its own crate to call the test-app binary with a specific service type argument. The test-app performs the following operations for each service type:

1. Creates a service object
2. Queries the initial status
3. Registers the service
4. Queries status after registration
5. Un registers the service
6. Queries status after un registration

## Code Signing

For LaunchAgents and LaunchDaemons, proper testing requires the application bundle to be code-signed. If you experience issues with these tests, make sure your test-app is properly signed.