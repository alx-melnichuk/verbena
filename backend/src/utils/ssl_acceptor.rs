use openssl::{pkey, ssl};
use std::{fs, io::Read};

pub fn create_ssl_acceptor_builder(
    path_certificate: &str,
    path_private_key: &str,
) -> ssl::SslAcceptorBuilder {
    // build TLS config from files
    let mut builder = ssl::SslAcceptor::mozilla_intermediate(ssl::SslMethod::tls()).unwrap();

    // set the encrypted private key // APP_PRIVATE_KEY
    builder.set_private_key(&load_encrypted_private_key(path_private_key)).unwrap();

    // set the unencrypted private key
    // (uncomment if you generate your own key+cert with `mkcert`, and also remove the statement above)
    // builder.set_private_key_file(path_private_key, ssl::SslFiletype::PEM).unwrap();

    // set the certificate chain file location // APP_CERTIFICATE
    builder.set_certificate_chain_file(&path_certificate).unwrap();

    builder
}

fn load_encrypted_private_key(path_private_key: &str) -> pkey::PKey<pkey::Private> {
    let mut file = fs::File::open(path_private_key).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");

    pkey::PKey::private_key_from_pem_passphrase(&buffer, b"password").unwrap()
}
