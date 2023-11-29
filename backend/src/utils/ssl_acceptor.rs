use openssl::{
    pkey::{PKey, Private},
    ssl::{SslAcceptor, SslAcceptorBuilder, SslMethod},
};
use std::fs::File;
use std::io::Read;

pub fn create_ssl_acceptor_builder(
    path_certificate: &str,
    path_private_key: &str,
) -> SslAcceptorBuilder {
    // build TLS config from files
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();

    // set the encrypted private key
    builder.set_private_key(&load_encrypted_private_key(path_private_key)).unwrap();

    // set the unencrypted private key
    // (uncomment if you generate your own key+cert with `mkcert`, and also remove the statement above)
    // builder
    //     .set_private_key_file("key.pem", openssl::ssl::SslFiletype::PEM)
    //     .unwrap();

    // set the certificate chain file location // "example.crt.pem"
    builder.set_certificate_chain_file(&path_certificate).unwrap();

    builder
}

fn load_encrypted_private_key(path_private_key: &str) -> PKey<Private> {
    let mut file = File::open(path_private_key).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");

    PKey::private_key_from_pem_passphrase(&buffer, b"password").unwrap()
}
