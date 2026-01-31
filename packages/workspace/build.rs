fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only compile workspace.proto - vdom and patches are compiled in evaluator
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .extern_path(
            ".paperclip.patches",
            "::paperclip_evaluator::vdom_differ::proto::patches",
        )
        .extern_path(
            ".paperclip.vdom",
            "::paperclip_evaluator::vdom_differ::proto::vdom",
        )
        .compile(&["../proto/src/workspace.proto"], &["../proto/src"])?;
    Ok(())
}
