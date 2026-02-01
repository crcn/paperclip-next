fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .build_client(false)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(
            &["../proto/src/vdom.proto", "../proto/src/patches.proto"],
            &["../proto/src"],
        )?;
    Ok(())
}
