use paperclip_workspace::WorkspaceServer;
use std::path::PathBuf;
use tonic::transport::Server;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get root directory from args or use current directory
    let root_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    println!("Starting Paperclip workspace server...");
    println!("Root directory: {:?}", root_dir);
    println!("Listening on 127.0.0.1:50051");

    let addr = "127.0.0.1:50051".parse()?;
    let workspace = WorkspaceServer::new(root_dir);

    Server::builder()
        .add_service(workspace.into_service())
        .serve(addr)
        .await?;

    Ok(())
}
