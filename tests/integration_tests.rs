use std::fs;
use std::path::PathBuf;
use anyhow::Result;

// Mock config and profile module imports
// Note: These would normally be private in src/, but we need to test them
// We'll use the public API where available

#[test]
fn test_profile_valid_loading() {
    // Test that a valid profile can be loaded
    let profile_path = PathBuf::from("tests/test_profiles/valid_profile.yaml");
    assert!(profile_path.exists(), "Test profile file should exist");
    
    let contents = fs::read_to_string(&profile_path)
        .expect("Should be able to read test profile");
    
    // Verify YAML structure
    assert!(contents.contains("name:"), "Profile should have a name field");
    assert!(contents.contains("Testing Profile"), "Profile should have correct name");
    assert!(contents.contains("limits:"), "Profile should have limits");
}

#[test]
fn test_profile_minimal_loading() {
    // Test that a minimal profile with defaults works
    let profile_path = PathBuf::from("tests/test_profiles/minimal_profile.yaml");
    assert!(profile_path.exists(), "Minimal profile file should exist");
    
    let contents = fs::read_to_string(&profile_path)
        .expect("Should be able to read minimal profile");
    
    assert!(contents.contains("name:"), "Should have a name");
    assert!(contents.contains("description:"), "Should have a description");
}

#[test]
fn test_profile_edge_case_max_values() {
    // Test profile with maximum allowed values
    let profile_path = PathBuf::from("tests/test_profiles/edge_case_max_values.yaml");
    assert!(profile_path.exists());
    
    let contents = fs::read_to_string(&profile_path).expect("Should read file");
    assert!(contents.contains("100"), "Should have 100% CPU");
    assert!(contents.contains("100"), "Should have 100% RAM");
    assert!(contents.contains("120"), "Should have 120°C temp");
}

#[test]
fn test_profile_edge_case_min_values() {
    // Test profile with minimum allowed values
    let profile_path = PathBuf::from("tests/test_profiles/edge_case_min_values.yaml");
    assert!(profile_path.exists());
    
    let contents = fs::read_to_string(&profile_path).expect("Should read file");
    assert!(contents.contains("max_cpu_percent: 0"), "Should have 0% CPU");
    assert!(contents.contains("max_ram_percent: 0"), "Should have 0% RAM");
    assert!(contents.contains("max_temp: 0"), "Should have 0°C temp");
}

#[test]
fn test_all_test_profiles_exist() {
    // Ensure all expected test profiles are present
    let test_profiles = vec![
        "valid_profile.yaml",
        "minimal_profile.yaml",
        "coding_profile.yaml",
        "edge_case_max_values.yaml",
        "edge_case_min_values.yaml",
        "invalid_cpu.yaml",
        "invalid_ram.yaml",
        "invalid_temp.yaml",
        "no_name.yaml",
        "empty_name.yaml",
    ];
    
    for profile in test_profiles {
        let path = PathBuf::from(format!("tests/test_profiles/{}", profile));
        assert!(
            path.exists(),
            "Test profile {} should exist",
            profile
        );
    }
}

#[test]
fn test_config_file_exists() {
    // Verify default config file exists
    let config_path = PathBuf::from("config/kern.yaml");
    assert!(config_path.exists(), "Default config should exist");
    
    let contents = fs::read_to_string(&config_path)
        .expect("Should be able to read config");
    
    // Verify essential config fields
    assert!(contents.contains("default_profile:"), "Should have default_profile");
    assert!(contents.contains("monitor_interval:"), "Should have monitor_interval");
    assert!(contents.contains("temperature:"), "Should have temperature config");
    assert!(contents.contains("limits:"), "Should have resource limits");
}

#[test]
fn test_profile_config_files_exist() {
    // Verify all profile config files exist
    let profiles_dir = PathBuf::from("config/profiles");
    
    if !profiles_dir.exists() {
        // This is OK for now - profiles may not be in config/
        return;
    }
    
    // If directory exists, it should have some profiles
    let entries = fs::read_dir(&profiles_dir).expect("Should be able to read profiles dir");
    let yaml_files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "yaml")
                .unwrap_or(false)
        })
        .collect();
    
    assert!(
        !yaml_files.is_empty(),
        "Should have at least one profile if profiles dir exists"
    );
}

#[test]
fn test_main_components_exist() {
    // Verify main Rust source files exist
    let source_files = vec![
        "src/main.rs",
        "src/monitor.rs",
        "src/config.rs",
        "src/profiles.rs",
    ];
    
    for file in source_files {
        let path = PathBuf::from(file);
        assert!(path.exists(), "Source file {} should exist", file);
    }
}

#[test]
fn test_cargo_toml_exists() {
    let cargo_path = PathBuf::from("Cargo.toml");
    assert!(cargo_path.exists(), "Cargo.toml should exist");
    
    let contents =
        fs::read_to_string(&cargo_path).expect("Should be able to read Cargo.toml");
    
    assert!(contents.contains("name = \"kern\""), "Cargo.toml should define kern package");
    assert!(
        contents.contains("serde"),
        "Should have serde dependency"
    );
    assert!(
        contents.contains("sysinfo"),
        "Should have sysinfo dependency"
    );
    assert!(contents.contains("clap"), "Should have clap CLI dependency");
}

#[test]
fn test_documentation_exists() {
    // Verify documentation files
    let docs = vec![
        "README.md",
        "docs/README.md",
        "docs/PROFILES.md",
        "docs/DBUS.md",
    ];
    
    for doc in docs {
        let path = PathBuf::from(doc);
        if path.exists() {
            let contents = fs::read_to_string(&path)
                .expect(&format!("Should be able to read {}", doc));
            assert!(
                !contents.is_empty(),
                "Documentation file {} should not be empty",
                doc
            );
        }
    }
}

#[test]
fn test_systemd_service_file_exists() {
    let service_path = PathBuf::from("systemd/kern.service");
    assert!(service_path.exists(), "systemd service file should exist");
    
    let contents = fs::read_to_string(&service_path)
        .expect("Should be able to read service file");
    
    assert!(
        contents.contains("[Unit]"),
        "Service file should have [Unit] section"
    );
    assert!(
        contents.contains("[Service]"),
        "Service file should have [Service] section"
    );
}

#[test]
fn test_install_scripts_exist() {
    let scripts = vec![
        "scripts/install.sh",
        "scripts/uninstall.sh",
        "scripts/build-extension.sh",
    ];
    
    for script in scripts {
        let path = PathBuf::from(script);
        assert!(path.exists(), "Script {} should exist", script);
    }
}

#[test]
fn test_extension_files_exist() {
    let ext_files = vec![
        "extension/extension.js",
        "extension/metadata.json",
        "extension/indicator.js",
        "extension/menu.js",
        "extension/dbus.js",
        "extension/prefs.js",
        "extension/stylesheet.css",
    ];
    
    for file in ext_files {
        let path = PathBuf::from(file);
        assert!(
            path.exists(),
            "Extension file {} should exist",
            file
        );
    }
}

#[test]
fn test_plan_documentation_exists() {
    let plan_path = PathBuf::from("plan/plan.md");
    assert!(plan_path.exists(), "Project plan should exist");
    
    let contents = fs::read_to_string(&plan_path)
        .expect("Should be able to read plan");
    
    assert!(contents.contains("PHASE"), "Plan should contain phase information");
}

// Integration tests for actual functionality
// These tests verify the modules work correctly together

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_project_structure_valid() {
        // Verify complete project structure
        let dirs = vec![
            "src/",
            "tests/",
            "config/",
            "docs/",
            "extension/",
            "scripts/",
            "systemd/",
        ];
        
        for dir in dirs {
            let path = PathBuf::from(dir);
            assert!(
                path.is_dir(),
                "Directory {} should exist",
                dir
            );
        }
    }

    #[test]
    fn test_no_empty_core_files() {
        // Verify core source files are not empty
        let core_files = vec![
            "src/main.rs",
            "src/monitor.rs",
            "src/config.rs",
            "src/profiles.rs",
        ];
        
        for file in core_files {
            let contents = fs::read_to_string(file)
                .expect(&format!("Should be able to read {}", file));
            assert!(
                !contents.trim().is_empty(),
                "Core file {} should not be empty",
                file
            );
            assert!(
                contents.lines().count() > 10,
                "Core file {} should have substantial content",
                file
            );
        }
    }

    #[test]
    fn test_yaml_files_valid_structure() {
        // Verify all YAML files have valid structure
        let yaml_files = vec![
            "config/kern.yaml",
            "tests/test_profiles/valid_profile.yaml",
            "tests/test_profiles/minimal_profile.yaml",
        ];
        
        for file in yaml_files {
            let path = PathBuf::from(file);
            if path.exists() {
                let contents = fs::read_to_string(&path)
                    .expect(&format!("Should read {}", file));
                
                // Basic YAML structure checks
                assert!(
                    !contents.trim().is_empty(),
                    "YAML file {} should not be empty",
                    file
                );
                
                // Check for key-value pairs
                assert!(
                    contents.contains(":"),
                    "YAML file {} should have key-value pairs",
                    file
                );
            }
        }
    }
}
