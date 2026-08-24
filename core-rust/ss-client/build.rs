fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Utilise l'exécutable protoc.exe pré-compilé fourni par protoc-bin-vendored
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);

    tonic_build::configure()
        .build_server(true) // Génère aussi le serveur pour le Mock Server de test
        .build_client(true)
        .compile(
            &["../../proto/framework.proto"],
            &["../../proto"],
        )?;
    println!("cargo:rerun-if-changed=../../proto/framework.proto");
    Ok(())
}
