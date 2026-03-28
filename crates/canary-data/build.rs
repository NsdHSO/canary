use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=data/");

    // Validate manufacturers.toml
    validate_toml_file("data/manufacturers.toml");

    // Validate universal data files
    validate_toml_files_in_dir("data/universal");
    validate_toml_files_in_dir("data/connectors");

    // Validate protocol files
    validate_toml_files_in_dir("data/protocols");

    // Validate DTC files
    validate_toml_files_in_dir("data/dtc");

    // Validate service procedure files
    validate_toml_files_in_dir("data/service_procedures");

    // Validate manufacturer data (before compression)
    validate_manufacturer_data();

    // Compress manufacturer files
    compress_manufacturer_files();

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

fn validate_manufacturer_data() {
    let manufacturers_dir = Path::new("data/manufacturers");
    if !manufacturers_dir.exists() {
        return;
    }

    for entry in fs::read_dir(manufacturers_dir).expect("Failed to read manufacturers") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.is_dir() {
            validate_toml_files_in_dir(path.to_str().unwrap());
        }
    }
}

fn compress_manufacturer_files() {
    println!("cargo:rerun-if-changed=data/manufacturers/");

    let manufacturers_dir = Path::new("data/manufacturers");
    if !manufacturers_dir.exists() {
        return; // No manufacturer data yet
    }

    // Iterate through manufacturer directories
    for entry in fs::read_dir(manufacturers_dir).expect("Failed to read manufacturers dir") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.is_dir() {
            compress_manufacturer_dir(&path);
        }
    }
}

fn compress_manufacturer_dir(dir: &Path) {
    for entry in fs::read_dir(dir).expect("Failed to read manufacturer dir") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            let toml_content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

            // Compress
            let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
            encoder.write_all(toml_content.as_bytes())
                .expect("Failed to compress");
            let compressed = encoder.finish().expect("Failed to finish compression");

            // Log compression ratio before moving compressed
            let ratio = compressed.len() as f32 / toml_content.len() as f32;

            // Write to .toml.gz
            let gz_path = path.with_extension("toml.gz");
            fs::write(&gz_path, compressed)
                .unwrap_or_else(|e| panic!("Failed to write {}: {}", gz_path.display(), e));
            println!("Compressed {}: {:.1}% of original",
                     path.file_name().unwrap().to_str().unwrap(),
                     ratio * 100.0);
        }
    }
}
