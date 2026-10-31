fn main() {
    for key in [
        "AEVRIK_CHANNEL",
        "GITHUB_REF",
        "GITHUB_SHA",
        "AEVRIK_BUILT_AT",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }
    let reference = std::env::var("GITHUB_REF").unwrap_or_default();
    let channel = match reference.as_str() {
        "refs/heads/dev" => "dev",
        "refs/heads/beta" => "beta",
        "refs/heads/master" => "stable",
        value if value.starts_with("refs/tags/v") => {
            if value.contains('-') {
                "beta"
            } else {
                "stable"
            }
        }
        _ => "local",
    };
    let channel = std::env::var("AEVRIK_CHANNEL").unwrap_or_else(|_| channel.into());
    assert!(
        matches!(channel.as_str(), "dev" | "beta" | "stable" | "local"),
        "Invalid release channel"
    );
    println!("cargo:rustc-env=AEVRIK_CHANNEL={channel}");
    println!(
        "cargo:rustc-env=AEVRIK_COMMIT={}",
        std::env::var("GITHUB_SHA").unwrap_or_default()
    );
    println!(
        "cargo:rustc-env=AEVRIK_BUILT_AT={}",
        std::env::var("AEVRIK_BUILT_AT").unwrap_or_else(|_| "0".into())
    );
    tauri_build::build()
}
