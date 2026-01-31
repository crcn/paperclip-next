fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile(
            &[
                "src/vdom.proto",
                "src/patches.proto",
                "src/workspace.proto",
            ],
            &["src/"],
        )?;
    Ok(())
}
