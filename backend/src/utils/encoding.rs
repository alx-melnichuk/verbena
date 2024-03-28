use openssl::{
    base64,
    encrypt::Encrypter,
    pkey::{PKey, Private},
    rsa::{Padding, Rsa},
};

pub fn encrypt_inform(source: &[u8]) -> Result<String, String> {
    // Generate a keypair
    let rsa = Rsa::generate(2048).unwrap();
    let keypair: PKey<Private> = PKey::from_rsa(rsa).unwrap();

    // Encrypt the data with RSA PKCS1
    let mut encrypter = Encrypter::new(&keypair).unwrap();
    encrypter.set_rsa_padding(Padding::PKCS1).unwrap();

    // Create an output buffer.
    let buffer_len = encrypter.encrypt_len(source).unwrap();
    let mut encrypted = vec![0; buffer_len];

    // Encrypt and truncate the buffer.
    let encrypted_len = encrypter.encrypt(source, &mut encrypted).map_err(|err| {
        eprintln!("err2: {:?}", &err);
        format!("{}: {:?}", &err.to_string(), &err)
    })?;

    // Shortens the vector, keeping the first `len` elements and dropping the rest.
    encrypted.truncate(encrypted_len);

    Ok(base64::encode_block(&encrypted))
}

#[cfg(test)]
mod tests {

    use super::*;

    // ** encrypt **

    #[test]
    fn test_encrypt_valid_data() {
        let source = "Test string for coding.";
        let result = encrypt_inform(source.as_bytes());
        assert!(result.is_ok());
        let receiver = result.unwrap_or("".to_string());
        assert!(receiver.len() > 0);
    }
}
