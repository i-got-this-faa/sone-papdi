//! Generate Rust protocol bindings from hyprwire XML protocol definitions.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    hyprwire_scanner::configure()
        .with_targets(hyprwire_scanner::Targets::ALL)
        .compile(&["protocols/sone-papdi-shell-v1.xml"])?;
    Ok(())
}
