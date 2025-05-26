use smappservice_rs::{AppService, ServiceStatus, ServiceType};
use std::env;

fn print_status(service_type: &str, operation: &str, status: ServiceStatus) {
    println!("[{}] {} status: {}", service_type, operation, status);
}

fn test_main_app() {
    println!("Testing MainApp service");

    // Create service
    let service = AppService::new(ServiceType::MainApp);

    // Query initial status
    let status = service.status();
    print_status("MainApp", "Initial", status);
    assert!(
        status == ServiceStatus::NotFound || status == ServiceStatus::NotRegistered,
        "MainApp should not be found initially"
    );

    // Register
    println!("[MainApp] Registering service...");
    let result = service.register();
    println!("[MainApp] Registration result: {:?}", result);
    assert!(result.is_ok(), "Failed to register MainApp service");

    // Query status after registration
    let status = service.status();
    print_status("MainApp", "After registration", status);
    assert_eq!(
        status,
        ServiceStatus::Enabled,
        "MainApp should be enabled after registration"
    );

    // Unregister
    println!("[MainApp] Unregistering service...");
    let result = service.unregister();
    println!("[MainApp] Unregistration result: {:?}", result);
    assert!(result.is_ok(), "Failed to unregister MainApp service");

    // Query status after unregistration
    let status = service.status();
    print_status("MainApp", "After unregistration", status);
    assert_eq!(
        status,
        ServiceStatus::NotRegistered,
        "MainApp should be not registered after unregistration"
    );
}

fn test_daemon() {
    println!("Testing Daemon service");

    // Create service
    let service = AppService::new(ServiceType::Daemon {
        plist_name: "com.example.smappservice-test-app.plist",
    });

    // Register
    println!("[Daemon] Registering service...");
    let result = service.register();
    println!("[Daemon] Registration result: {:?}", result);
    if let Err(err) = &result {
        if err == &smappservice_rs::ServiceManagementError::InvalidSignature {
            println!("[Daemon] Invalid signature error. Please sign the app bundle using the following command:");
            println!("[Daemon] codesign --deep --force --verify --options runtime --sign \"Your Signature Title\" test-app/target/debug/bundle/osx/smappservice-test-app.app");
        }
        panic!("Failed to register Daemon service: {:?}", err);
    }

    // Query status after registration
    let status = service.status();
    print_status("Daemon", "After registration", status);
    if status == ServiceStatus::RequiresApproval {
        println!("[Daemon] Requires approval. Please approve the service in System Preferences.");
    }
    assert_eq!(
        status,
        ServiceStatus::Enabled,
        "Daemon should be enabled after registration"
    );

    // Unregister
    println!("[Daemon] Unregistering service...");
    let result = service.unregister();
    println!("[Daemon] Unregistration result: {:?}", result);
    assert!(result.is_ok(), "Failed to unregister Daemon service");

    // Query status after unregistration
    let status = service.status();
    print_status("Daemon", "After unregistration", status);
    assert_eq!(
        status,
        ServiceStatus::NotRegistered,
        "Daemon should be not registered after unregistration"
    );
}

fn test_login_item() {
    println!("Testing LoginItem service");

    // Create service
    let service = AppService::new(ServiceType::LoginItem {
        identifier: "com.example.smappservice-test-app",
    });

    // Query initial status
    let status = service.status();
    print_status("LoginItem", "Initial", status);
    assert!(
        status == ServiceStatus::NotFound || status == ServiceStatus::NotRegistered,
        "LoginItem should not be found initially"
    );

    // Register
    println!("[LoginItem] Registering service...");
    let result = service.register();
    println!("[LoginItem] Registration result: {:?}", result);
    assert!(result.is_ok(), "Failed to register LoginItem service");

    // Query status after registration
    let status = service.status();
    print_status("LoginItem", "After registration", status);
    assert_eq!(
        status,
        ServiceStatus::Enabled,
        "LoginItem should be enabled after registration"
    );

    // Unregister
    println!("[LoginItem] Unregistering service...");
    let result = service.unregister();
    println!("[LoginItem] Unregistration result: {:?}", result);
    assert!(result.is_ok(), "Failed to unregister LoginItem service");

    // Query status after unregistration
    let status = service.status();
    print_status("LoginItem", "After unregistration", status);
    assert_eq!(
        status,
        ServiceStatus::NotRegistered,
        "LoginItem should be not registered after unregistration"
    );
}

fn test_agent() {
    println!("Testing Agent service");

    // Create service
    let service = AppService::new(ServiceType::Agent {
        plist_name: "com.example.smappservice-test-app.plist",
    });

    // Query initial status
    let status = service.status();
    print_status("Agent", "Initial", status);
    assert!(
        status == ServiceStatus::NotFound || status == ServiceStatus::NotRegistered,
        "Agent should not be found initially"
    );

    // Register
    println!("[Agent] Registering service...");
    let result = service.register();
    println!("[Agent] Registration result: {:?}", result);
    if let Err(err) = &result {
        if err == &smappservice_rs::ServiceManagementError::InvalidSignature {
            println!("[Agent] Invalid signature error. Please sign the app bundle using the following command:");
            println!("[Agent] codesign --deep --force --verify --options runtime --sign \"Your Signature Title\" test-app/target/debug/bundle/osx/smappservice-test-app.app");
        }
    }
    assert!(result.is_ok(), "Failed to register Agent service");

    // Query status after registration
    let status = service.status();
    print_status("Agent", "After registration", status);
    assert_eq!(
        status,
        ServiceStatus::Enabled,
        "Agent should be enabled after registration"
    );

    // Unregister
    println!("[Agent] Unregistering service...");
    let result = service.unregister();
    println!("[Agent] Unregistration result: {:?}", result);
    assert!(result.is_ok(), "Failed to unregister Agent service");

    // Query status after unregistration
    let status = service.status();
    print_status("Agent", "After unregistration", status);
    assert_eq!(
        status,
        ServiceStatus::NotRegistered,
        "Agent should be not registered after unregistration"
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} [mainapp|daemon|loginitem|agent]", args[0]);
        return;
    }

    let service_type = &args[1].to_lowercase();

    match service_type.as_str() {
        "mainapp" => test_main_app(),
        "daemon" => test_daemon(),
        "loginitem" => test_login_item(),
        "agent" => test_agent(),
        _ => {
            eprintln!("Usage: {} [mainapp|daemon|loginitem|agent]", args[0]);
        }
    }
}
