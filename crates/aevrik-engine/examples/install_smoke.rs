//! Opt-in network verification: cargo run -p aevrik-engine --example install_smoke -- <isolated-dir>
use aevrik_engine::packages::PackageManager;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::args_os()
        .nth(1)
        .ok_or("Provide an isolated installation directory.")?;
    let manager = PackageManager::new(std::path::PathBuf::from(root))?;
    let requested: Vec<String> = std::env::args().skip(2).collect();
    let ids = if requested.is_empty() {
        vec![
            "caddy".into(),
            "php84".into(),
            "node24".into(),
            "mariadb118".into(),
        ]
    } else {
        requested
    };
    for id in ids {
        println!("Installing {id} from the pinned official source...");
        manager.install(&id)?;
        println!("Installed {id}");
    }
    println!("{}", serde_json::to_string_pretty(&manager.list()?)?);
    Ok(())
}
