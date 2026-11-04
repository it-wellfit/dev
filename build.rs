#![deny(warnings)]
fn main() {
    println!("🔨 Building Flaps ERP workspace...");

    // Ensure target directory exists
    let target_dir = std::path::Path::new("target");
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir).expect("Failed to create target directory");
        println!("✅ Created target directory");
    }

    // Create subdirectories
    let build_dir = target_dir.join("build");
    if !build_dir.exists() {
        std::fs::create_dir_all(&build_dir).expect("Failed to create build directory");
        println!("✅ Created build directory");
    }

    // Create release directory
    let release_dir = target_dir.join("release");
    if !release_dir.exists() {
        std::fs::create_dir_all(&release_dir).expect("Failed to create release directory");
        println!("✅ Created release directory");
    }

    // Create debug directory
    let debug_dir = target_dir.join("debug");
    if !debug_dir.exists() {
        std::fs::create_dir_all(&debug_dir).expect("Failed to create debug directory");
        println!("✅ Created debug directory");
    }

    // Create data directory
    let data_dir = std::path::Path::new("data");
    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir).expect("Failed to create data directory");
        println!("✅ Created data directory");
    }

    // Create search index directory
    let search_dir = data_dir.join("search_index");
    if !search_dir.exists() {
        std::fs::create_dir_all(&search_dir).expect("Failed to create search index directory");
        println!("✅ Created search index directory");
    }

    println!("🎉 Workspace build completed successfully!");
}