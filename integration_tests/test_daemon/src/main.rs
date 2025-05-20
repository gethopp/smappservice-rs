use std::io;
use std::path::PathBuf;
use std::process::{Command, Output};

fn main() {
    println!("Running daemon integration test");

    match run_test_app() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            println!("TEST OUTPUT:");
            println!("{}", stdout);

            if !stderr.is_empty() {
                eprintln!("ERRORS:");
                eprintln!("{}", stderr);
            }

            if output.status.success() {
                println!("Test completed successfully!");
            } else {
                eprintln!("Test failed with exit code: {:?}", output.status.code());
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to run test app: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_test_app() -> io::Result<Output> {
    // Get the path to the test app binary
    let test_app_path = get_test_app_path()?;

    // Create and execute the command with 'daemon' argument
    let output = Command::new(&test_app_path).arg("daemon").output()?;

    Ok(output)
}

fn get_test_app_path() -> io::Result<PathBuf> {
    // Get the project root directory (where the integration_tests directory is)
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    // Path to the test app executable
    let test_app_path = root_dir.join("integration_tests/test-app/target/debug/bundle/osx/smappservice-test-app.app/Contents/MacOS/smappservice-test-app");

    let test_app_dir = root_dir.join("integration_tests/test-app");

    if !test_app_path.exists() {
        // Build the test app
        println!("Building test app...");
        let build_status = Command::new("cargo")
            .current_dir(&test_app_dir)
            .arg("bundle")
            .status()?;
        println!("build_status: {:?}", build_status);
    }

    // Create LaunchDaemons directory if it doesn't exist
    let launch_daemons_dir = test_app_dir
        .join("target/debug/bundle/osx/smappservice-test-app.app/Contents/Library/LaunchDaemons");
    if !launch_daemons_dir.exists() {
        println!("Creating LaunchDaemons directory...");
        std::fs::create_dir_all(&launch_daemons_dir)?;
    }

    // Create the plist file for the daemon
    let plist_path = launch_daemons_dir.join("com.example.smappservice-test-app.plist");
    println!("Creating plist file at: {:?}", plist_path);

    // Use the test app binary path for the ProgramArguments
    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.example.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
        test_app_path.to_str().unwrap()
    );

    std::fs::write(&plist_path, plist_content)?;
    println!("Plist file created successfully.");

    Ok(test_app_path)
}
