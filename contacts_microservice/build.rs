fn main() {
    let auth = "./proto/auth_service.proto";
    let address_book = "./proto/address_book_service.proto";

    tonic_build::configure()
        .build_server(true)
        .compile(&[auth,address_book], &["."])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));
}