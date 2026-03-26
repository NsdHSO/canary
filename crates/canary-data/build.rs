use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=data/");

    // Validate manufacturers.toml
    validate_toml_file("data/manufacturers.toml");

    // Validate pinout files
    validate_toml_files_in_dir("data/pinouts");

    // Validate protocol files
    validate_toml_files_in_dir("data/protocols");

    // Validate DTC files
    validate_toml_files_in_dir("data/dtc");

    // Validate service procedure files
    validate_toml_files_in_dir("data/service_procedures");

    println!("All data files validated successfully!");
}

fn validate_toml_file(path: &str) {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));

    toml::from_str::<toml::Value>(&content)
        .unwrap_or_else(|e| panic!("Invalid TOML in {}: {}", path, e));

    println!("✓ Validated: {}", path);
}

fn validate_toml_files_in_dir(dir: &str) {
    if !Path::new(dir).exists() {
        return;
    }

    visit_toml_files(Path::new(dir));
}

fn visit_toml_files(dir: &Path) {
    if dir.is_dir() {
        fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("Failed to read directory {:?}: {}", dir, e))
            .filter_map(Result::ok)
            .for_each(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    visit_toml_files(&path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    validate_toml_file(path.to_str().unwrap());
                }
            });
    }
}
