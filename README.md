# SMAppService-RS

<!-- [![Crates.io](https://img.shields.io/crates/v/smappservice-rs)](https://crates.io/crates/smappservice-rs)
[![Documentation](https://docs.rs/smappservice-rs/badge.svg)](https://docs.rs/smappservice-rs) -->
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust wrapper for macOS's ServiceManagement framework, specifically the SMAppService API. This library provides a safe and idiomatic Rust interface for registering and managing macOS services.

## Overview

The ServiceManagement framework in macOS provides a way for applications to manage system services. This library wraps the `objc2-service-management` API in a Rust-friendly interface, allowing to:

- Register applications as login items
- Register and manage launch agents and daemons from the application bundle
- Check the status of registered services
- Open the System Settings Login Items panel

## Requirements

- macOS 13.0 (Ventura) or later

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
smappservice-rs = "0.1.0"
```

## Usage

### Register the Main Application as a Login Item

```rust
use smappservice_rs::{AppService, ServiceType};

let app_service = AppService::new(ServiceType::MainApp);
match app_service.register() {
    Ok(()) => println!("Application registered successfully as login item!"),
    Err(e) => eprintln!("Failed to register application: {}", e),
}
```

### Check Registration Status

```rust
use smappservice_rs::{AppService, ServiceType, ServiceStatus};

let app_service = AppService::new(ServiceType::MainApp);
let status = app_service.status();
println!("Service status: {}", status);

if status == ServiceStatus::RequiresApproval {
    println!("Please approve the service in System Settings");
    AppService::open_system_settings_login_items();
}
```

### Register a LaunchAgent

```rust
use smappservice_rs::{AppService, ServiceType};

let agent_service = AppService::new(ServiceType::Agent {
    plist_name: "com.example.myapp.agent.plist"
});
if let Err(e) = agent_service.register() {
    eprintln!("Failed to register agent: {}", e);
}
```

### Register a LaunchDaemon

```rust
use smappservice_rs::{AppService, ServiceType};

let daemon_service = AppService::new(ServiceType::Daemon {
    plist_name: "com.example.myapp.daemon.plist"
});
if let Err(e) = daemon_service.register() {
    eprintln!("Failed to register daemon: {}", e);
}
```

### Register a Helper Application as a Login Item

```rust
use smappservice_rs::{AppService, ServiceType};

let login_item = AppService::new(ServiceType::LoginItem {
    identifier: "com.example.helper"
});
if let Err(e) = login_item.register() {
    eprintln!("Failed to register login item: {}", e);
}
```

### Unregister a Service

```rust
use smappservice_rs::{AppService, ServiceType};

let app_service = AppService::new(ServiceType::MainApp);
match app_service.unregister() {
    Ok(()) => println!("Application unregistered successfully!"),
    Err(e) => eprintln!("Failed to unregister application: {}", e),
}
```

## Testing

Due to the nature of the ServiceManagement framework, testing is primarily done through integration tests. The tests are located in the [integration_tests](integration_tests/) directory and cover various service types.

## License

This project is licensed under the MIT License - see the LICENSE.md file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request or open an Issue.