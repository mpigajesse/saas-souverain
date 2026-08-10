fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Régénération automatique des bindings Rust depuis proto/framework.proto
    tonic_build::configure()
        .build_server(false) // Mission B a seulement besoin du client gRPC
        .build_client(true)
        .compile(
            &["../../proto/framework.proto"],
            &["../../proto"],
        )?;
    println!("cargo:rerun-if-changed=../../proto/framework.proto");
    Ok(())
}
