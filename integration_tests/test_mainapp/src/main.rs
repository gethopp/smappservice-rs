use std::io;
use std::path::PathBuf;
use std::process::{Command, Output};

fn main() {
    println!("Running mainapp integration test");

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
            }
        }
        Err(e) => {
            eprintln!("Failed to run test app: {}", e);
        }
    }
}

fn run_test_app() -> io::Result<Output> {
    // Get the path to the test app binary
    let test_app_path = get_test_app_path()?;

    // Create and execute the command with 'mainapp' argument
    let output = Command::new(&test_app_path).arg("mainapp").output()?;

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

    if !test_app_path.exists() {
        // Path to the test app source directory
        let test_app_dir = root_dir.join("integration_tests/test-app");
        // Path to the test app binary
        let build_status = Command::new("cargo")
            .current_dir(&test_app_dir)
            .arg("bundle")
            .status()?;
        println!("build_status: {:?}", build_status);
    }

    Ok(test_app_path)
}
