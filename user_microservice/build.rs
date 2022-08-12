fn main() {
    let user = "./proto/user_service.proto";
    let auth = "./proto/auth_service.proto";

    tonic_build::configure()
        .build_server(true)
        .compile(&[user, auth], &["."])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));    
}