// use std::str;
use openssl::{
    base64, encrypt, error,
    pkey::{PKey, Private},
    rsa::Padding,
};
/** Deserializes a private key from a PEM-encoded key type specific format. */
fn pr_key_from_pem(pr_key_pem: &str) -> Result<PKey<Private>, error::ErrorStack> {
    let pr_key_pem_buf: &[u8] = &pr_key_pem.as_bytes();
    PKey::private_key_from_pem(pr_key_pem_buf)
}
// https://docs.rs/openssl/0.10.64/openssl/encrypt/index.html
/** Encrypt data with the specified private key.
 * pr_key_pem: `&str` - Private key in PEM encoding format.
 * data: `&[u8]` - Data buffer for encryption.
 * Returns: encrypted data as a base64 `String`.
*/
pub fn encrypt_utf8(pr_key_pem: &str, data: &[u8]) -> Result<String, String> {
    // Deserializes a private key from a PEM-encoded key type specific format.
    let keypair: PKey<Private> = pr_key_from_pem(pr_key_pem).map_err(|err| err.to_string())?;

    // Encrypt the data with RSA PKCS1
    let mut encrypter = encrypt::Encrypter::new(&keypair).map_err(|err| err.to_string())?;
    encrypter.set_rsa_padding(Padding::PKCS1).map_err(|err| err.to_string())?;

    // Create an output buffer
    let buffer_len = encrypter.encrypt_len(data).map_err(|err| err.to_string())?;
    // Initialize the entire vector to zero.
    let mut encrypted = vec![0; buffer_len];

    // Encrypt and truncate the buffer
    let encrypted_len = encrypter.encrypt(data, &mut encrypted).map_err(|err| err.to_string())?;
    // Trim the buffer to the length of the resulting data.
    encrypted.truncate(encrypted_len);
    // Encodes a slice of bytes to a base64 string.
    Ok(base64::encode_block(&encrypted))
}
// https://docs.rs/openssl/0.10.64/openssl/encrypt/index.html
/** Decrypt data with the specified private key.
 * pr_key_pem: `&str` - Private key in PEM encoding format.
 * encrypted: `&str` - An encrypted message string in Base64 encoding.
 * Returns: the decrypted data as a `Vec<u8>`.
 */
pub fn decrypt_utf8(pr_key_pem: &str, encrypted: &str) -> Result<Vec<u8>, String> {
    // Decodes a base64-encoded string to bytes.
    let encrypted = base64::decode_block(encrypted).map_err(|err| err.to_string())?;
    // Deserializes a private key from a PEM-encoded key type specific format.
    let keypair: PKey<Private> = pr_key_from_pem(pr_key_pem).map_err(|err| err.to_string())?;

    // Decrypt the data
    let mut decrypter = encrypt::Decrypter::new(&keypair).map_err(|err| err.to_string())?;
    decrypter.set_rsa_padding(Padding::PKCS1).map_err(|err| err.to_string())?;
    // Create an output buffer
    let buffer_len = decrypter.decrypt_len(&encrypted).map_err(|err| err.to_string())?;
    let mut decrypted = vec![0; buffer_len];
    // Encrypt and truncate the buffer
    let decrypted_len = decrypter.decrypt(&encrypted, &mut decrypted).map_err(|err| err.to_string())?;
    decrypted.truncate(decrypted_len);

    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;
    /** Get the private key in PEM encoded format. */
    fn get_pr_key_pem() -> String {
        let mut result: Vec<String> = Vec::new();
        result.push("-----BEGIN PRIVATE KEY-----\n".to_string());
        result.push("MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDdf1ULH7RbBAsR\n".to_string());
        result.push("EWz137GnWOothrXA+BitZ2kJ//N6Xx6+xSwjQ/lzr7RubYxh9NVnYQm3ulFuBhVA\n".to_string());
        result.push("1eRMCJhE3U5GhjrBc7btvPWHp0MRv0LSiBZ43HItE/GUDL+Gi0tjCJnNQemsJXs7\n".to_string());
        result.push("9QrG48nqpZ1+0W4yEsnaoYVLRfnqGsNYJMRcJy3G1wDfSnUekhOQd///LRp/GRUb\n".to_string());
        result.push("n/+YP5lE0khgbox/xAK/9q5HsJ0LN6tNIBBmqnT/piYXjEJp2ay11MMiuvd8Y0UK\n".to_string());
        result.push("kiW+7xKmWpDQ620ZeuJNX5Ao2kMxc/OnNs8su9+IdUrZNEtFVOTDrEI73Q3kzD7C\n".to_string());
        result.push("SF8ypLcHAgMBAAECggEAEdHTLTRtRZ9DJqngur025TA9ktESIUa0cYruw+uwEmnB\n".to_string());
        result.push("pvZk8SdgU8LQgMvWbfENFCaV6g6Wy4O4xQEQP1+3pF6rk/frlTGyh1ja5gF6X1yk\n".to_string());
        result.push("SxzAthOCKgc/Obp21COzunFwMje5zWwsiGvT5KFh1rthOtkthODHFYxGFYK5MvCy\n".to_string());
        result.push("6Sr2OCJEMtE+sqWChH8fIGItPIGKqkaF7VrN0+rzY5PqyKSf90OJwIQJ/a1ReNth\n".to_string());
        result.push("vidvcBsWCdiSF01lYCafbBv3cBRbri4GGruaZ9e224MPOOAx4175qxOYC33eL90Z\n".to_string());
        result.push("0TO74JGxuq2OXCxYw/KiuvOIfQa9O8EhSD0bFS3ZEQKBgQDjxJYOW+Wm9k6qXuyC\n".to_string());
        result.push("D2TNxtHdeFSze3T+Mj8HIl1SJH3dlOAajAYJYlR3htP4508M0+UePrJtyupdNF3C\n".to_string());
        result.push("F9dJtu/p/UhUD07tD+ULujjO86sS36/GyXtGxNV8nINKqNpSEe2HAQjJGyP8x9Xt\n".to_string());
        result.push("eVxNHk2A6X3Dagp0umd83v73MQKBgQD488Xl2JzKqhrfYUIYiuKdecNh+kr08icH\n".to_string());
        result.push("gHFZPeNIVrgtfjbyI7zCbKvTxBNg3EZMfRsdS1CbiowPMxPcYEjDDFMQJ9AE3yvI\n".to_string());
        result.push("B2lWixploEmjkx2OyPPhQQ4vdmXLnC14AaGxzyDUOuemZn9pBc2vuXVokAAQdUC1\n".to_string());
        result.push("A2q8rwdztwKBgE9Q2AhsDA8WWtKNd8La5XmbMN3011ohNd6HVNeBKgo+1u3guCHG\n".to_string());
        result.push("fRureEqfUxWsRyTqbTEZGD2PmgmXAMdkUf5DjExpfVR4eD2peVOaJ8o5pGtQJgAN\n".to_string());
        result.push("jbZZORbJ0hafslc+Ev8eZxbRMrkGRgMKbhAU61xm8vqn5Lg9aWhcp2EhAoGAIvSH\n".to_string());
        result.push("ivhZO5Oa5laPo0aM/zODnZQ5Rh9iH4mHYNJxwUx729dm+6TM8je0AK39UpJbRI4k\n".to_string());
        result.push("an6SuORBOjkfxse2L7zhRNlyOdzkFtgDkGVDtZVGAbO8aLoKlExAI6XqMSais8D2\n".to_string());
        result.push("5TKCF4qV0CWAKkzoTo4p0B64A5eTGFd8ezXQRA0CgYEA0J5gCYiKmA8+5NBw+PUr\n".to_string());
        result.push("6ptrmJnGQhrfcwnF8diRFVpCXvbXR8Q0KTlHa8y8TxPtMYou8YcauDb9ppcQike3\n".to_string());
        result.push("X4MUEvfi2jU/WvpjBZCvXe4bC0Bj5IXhYm0mw5OXpwjJ4DevUGShGXtOZyVXv3NT\n".to_string());
        result.push("W5N93UmbAj+n5lu17tKxnYQ=\n".to_string());
        result.push("-----END PRIVATE KEY-----\n".to_string());
        result.join("")
    }

    // ** encrypt **
    #[test]
    fn test_encrypt_utf8_bad_pr_key_pem() {
        let data = "Test data1 string.".as_bytes();
        let pr_key_pem_str = "".to_string();
        // Encrypt data with the specified private key.
        let result = encrypt_utf8(&pr_key_pem_str, data);

        assert!(result.is_err());
        let err = format!(
            "error:{} routines:{}:../crypto/encode_decode/decoder_lib.c:101:{}",
            "1E08010C:DECODER", "OSSL_DECODER_from_bio:unsupported", "No supported data to decode. Input type: PEM"
        );
        assert_eq!(result.unwrap_err(), err);
    }
    #[test]
    fn test_encrypt_utf8_empty_data() {
        // Get the private key in PEM encoded format.
        let pr_key_pem_str = get_pr_key_pem();
        // Deserializes a private key from a PEM-encoded key type specific format.
        let keypair = PKey::private_key_from_pem(&pr_key_pem_str.as_bytes()).unwrap();

        let data = "".as_bytes();

        // Encrypt data with the specified private key.
        let encrypted_base64 = encrypt_utf8(&pr_key_pem_str, data).unwrap();

        // Decodes a base64-encoded string to bytes.
        let encrypted2 = base64::decode_block(&encrypted_base64).unwrap();

        // Decrypt the data
        let mut decrypter = encrypt::Decrypter::new(&keypair).unwrap();
        decrypter.set_rsa_padding(Padding::PKCS1).unwrap();
        // Create an output buffer
        let buffer_len = decrypter.decrypt_len(&encrypted2).unwrap();
        let mut decrypted = vec![0; buffer_len];
        // Encrypt and truncate the buffer
        let decrypted_len = decrypter.decrypt(&encrypted2, &mut decrypted).unwrap();
        decrypted.truncate(decrypted_len);

        assert_eq!(&*decrypted, data);
    }
    #[test]
    fn test_encrypt_utf8_valid_data() {
        // Get the private key in PEM encoded format.
        let pr_key_pem_str = get_pr_key_pem();
        // Deserializes a private key from a PEM-encoded key type specific format.
        let keypair = PKey::private_key_from_pem(&pr_key_pem_str.as_bytes()).unwrap();

        let data = "Test data2 string.".as_bytes();

        // Encrypt data with the specified private key.
        let encrypted_base64 = encrypt_utf8(&pr_key_pem_str, data).unwrap();

        // Decodes a base64-encoded string to bytes.
        let encrypted = base64::decode_block(&encrypted_base64).unwrap();

        // Decrypt the data
        let mut decrypter = encrypt::Decrypter::new(&keypair).unwrap();
        decrypter.set_rsa_padding(Padding::PKCS1).unwrap();
        // Create an output buffer
        let buffer_len = decrypter.decrypt_len(&encrypted).unwrap();
        let mut decrypted = vec![0; buffer_len];
        // Encrypt and truncate the buffer
        let decrypted_len = decrypter.decrypt(&encrypted, &mut decrypted).unwrap();
        decrypted.truncate(decrypted_len);

        assert_eq!(&*decrypted, data);
    }

    // ** decrypt **
    #[test]
    fn test_decrypt_utf8_bad_pr_key_pem() {
        // Get the private key in PEM encoded format.
        let pr_key_pem_str = get_pr_key_pem();
        // Deserializes a private key from a PEM-encoded key type specific format.
        let keypair = PKey::private_key_from_pem(&pr_key_pem_str.as_bytes()).unwrap();

        let data = "Test data3 string.".as_bytes();

        // Encrypt the data with RSA PKCS1
        let mut encrypter = encrypt::Encrypter::new(&keypair).unwrap();
        encrypter.set_rsa_padding(Padding::PKCS1).unwrap();
        // Create an output buffer
        let buffer_len = encrypter.encrypt_len(data).unwrap();
        let mut encrypted = vec![0; buffer_len];
        // Encrypt and truncate the buffer
        let encrypted_len = encrypter.encrypt(data, &mut encrypted).unwrap();
        encrypted.truncate(encrypted_len);
        // Encodes a slice of bytes to a base64 string.
        let encrypted_base64: String = base64::encode_block(&encrypted);

        let pr_key_pem_str2 = "".to_string();
        // Decrypt data with the specified private key.
        let result = decrypt_utf8(&pr_key_pem_str2, &encrypted_base64);

        assert!(result.is_err());
        let err = format!(
            "error:{} routines:{}:../crypto/encode_decode/decoder_lib.c:101:{}",
            "1E08010C:DECODER", "OSSL_DECODER_from_bio:unsupported", "No supported data to decode. Input type: PEM"
        );
        assert_eq!(result.unwrap_err(), err);
    }
    #[test]
    fn test_decrypt_utf8_empty_data() {
        // Get the private key in PEM encoded format.
        let pr_key_pem_str = get_pr_key_pem();
        // Deserializes a private key from a PEM-encoded key type specific format.
        let keypair = PKey::private_key_from_pem(&pr_key_pem_str.as_bytes()).unwrap();

        let data = "".as_bytes();

        // Encrypt the data with RSA PKCS1
        let mut encrypter = encrypt::Encrypter::new(&keypair).unwrap();
        encrypter.set_rsa_padding(Padding::PKCS1).unwrap();
        // Create an output buffer
        let buffer_len = encrypter.encrypt_len(data).unwrap();
        let mut encrypted = vec![0; buffer_len];
        // Encrypt and truncate the buffer
        let encrypted_len = encrypter.encrypt(data, &mut encrypted).unwrap();
        encrypted.truncate(encrypted_len);
        // Encodes a slice of bytes to a base64 string.
        let encrypted_base64: String = base64::encode_block(&encrypted);

        // Decrypt data with the specified private key.
        let decrypted: Vec<u8> = decrypt_utf8(&pr_key_pem_str, &encrypted_base64).unwrap();

        assert_eq!(&*decrypted, data);
    }
    #[test]
    fn test_decrypt_utf8_valid_data() {
        // Get the private key in PEM encoded format.
        let pr_key_pem_str = get_pr_key_pem();
        // Deserializes a private key from a PEM-encoded key type specific format.
        let keypair = PKey::private_key_from_pem(&pr_key_pem_str.as_bytes()).unwrap();

        let data = "Test data4 string.".as_bytes();

        // Encrypt the data with RSA PKCS1
        let mut encrypter = encrypt::Encrypter::new(&keypair).unwrap();
        encrypter.set_rsa_padding(Padding::PKCS1).unwrap();
        // Create an output buffer
        let buffer_len = encrypter.encrypt_len(data).unwrap();
        let mut encrypted = vec![0; buffer_len];
        // Encrypt and truncate the buffer
        let encrypted_len = encrypter.encrypt(data, &mut encrypted).unwrap();
        encrypted.truncate(encrypted_len);
        // Encodes a slice of bytes to a base64 string.
        let encrypted_base64: String = base64::encode_block(&encrypted);

        // Decrypt data with the specified private key.
        let decrypted: Vec<u8> = decrypt_utf8(&pr_key_pem_str, &encrypted_base64).unwrap();

        assert_eq!(&*decrypted, data);
    }
}
