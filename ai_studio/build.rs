fn main() {
    let auth = "./proto/auth_service.proto";
    let workspace = "./proto/workspace_service.proto";

    tonic_build::configure()
        .build_server(true)
        .compile(&[auth, workspace], &["."])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));
}