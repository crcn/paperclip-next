fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .build_client(false)
        .compile(
            &["../proto/src/vdom.proto", "../proto/src/patches.proto"],
            &["../proto/src"],
        )?;
    Ok(())
}
