use openssl::{
    base64, encrypt, hash,
    pkey::{PKey, Private},
    rsa::Padding,
    symm,
};

pub const CRT_MESSAGE_STRING_EMPTY: &str = "The encrypted message string is empty.";
pub const CRT_WRONG_STRING_BASE64URL: &str =
    "Base64Url must contain: \"A-Za-z0-9\\-_\" and have a length that is a multiple of 4.";

// https://docs.rs/openssl/0.10.64/openssl/encrypt/index.html
/// Encrypt data with the specified private key.
/// * pr_key_pem: `&str` - Private key in PEM encoding format.
/// * data: `&[u8]` - Data buffer for encryption.
/// * Returns: `String` - encrypted data as a base64.
pub fn encrypt_utf8(pr_key_pem: &str, data: &[u8]) -> Result<String, String> {
    // Deserializes a private key from a PEM-encoded key type specific format.
    let pr_key_pem_buf: &[u8] = &pr_key_pem.as_bytes();
    let keypair: PKey<Private> = PKey::private_key_from_pem(pr_key_pem_buf).map_err(|err| err.to_string())?;

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
/// Decrypt data with the specified private key.
/// * pr_key_pem: `&str` - Private key in PEM encoding format.
/// * data: `&str` - An encrypted message string in Base64 encoding.
/// * Returns: `Vec<u8>` - the decrypted data.
pub fn decrypt_utf8(pr_key_pem: &str, data: &str) -> Result<Vec<u8>, String> {
    // Decodes a base64-encoded string to bytes.
    let encrypted = base64::decode_block(data).map_err(|err| err.to_string())?;
    // Deserializes a private key from a PEM-encoded key type specific format.
    let pr_key_pem_buf: &[u8] = &pr_key_pem.as_bytes();
    let keypair: PKey<Private> = PKey::private_key_from_pem(pr_key_pem_buf).map_err(|err| err.to_string())?;

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
/// Check the characters of a base64 URL string.
fn check_symbols_base64url(src: &str) -> bool {
    let buf = "=-_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    src.chars().all(|ch| buf.contains(ch))
}
/// Encodes the string into a base64 URL.
fn encode_base64url(data: &str) -> String {
    data.replace("+", "-").replace("/", "_")
}
/// Decodes the string into a base64 URL.
fn decode_base64url(data: &str) -> String {
    data.replace("-", "+").replace("_", "/")
}
/// Encrypt the data using a secret string.
/// * secret: `&[u8]` - Secret string
/// * data: `&[u8]` - Data buffer for encryption.
/// * Returns: `String` - encrypted data as a base64.
pub fn encrypt_aes(secret: &[u8], data: &[u8]) -> Result<String, String> {
    // Get the key from the secret string.
    let digest = hash::hash(hash::MessageDigest::md5(), secret).map_err(|err| err.to_string())?;
    let key = digest.as_ref();

    // Get the reverse array for "secret".
    let secret_rv: Vec<u8> = secret.iter().copied().rev().collect();
    // Get the 'init_vec' from the secret string.
    let digest = hash::hash(hash::MessageDigest::md5(), &secret_rv).map_err(|err| err.to_string())?;
    let iv = digest.as_ref();

    let cipher = symm::Cipher::aes_128_cbc();
    // Encrypts data in one go, and returns the encrypted data.
    let encrypted = symm::encrypt(cipher, &key, Some(&iv), data).map_err(|e| e.to_string())?;
    // Encodes a slice of bytes to a base64 string.
    let encrypted2 = base64::encode_block(&encrypted);
    // Encodes the string into a base64 URL.
    Ok(encode_base64url(&encrypted2))
}
/// Decrypt the data using the secret string.
/// * secret: `&[u8]` - Secret string
/// * data: `&str` - An encrypted message string in Base64 encoding.
/// * Returns: `Vec<u8>` - the decrypted data.
pub fn decrypt_aes(secret: &[u8], data: &str) -> Result<Vec<u8>, String> {
    let data_len = data.len();
    if data_len == 0 {
        return Err(CRT_MESSAGE_STRING_EMPTY.to_string());
    }
    if data_len % 4 != 0 || !check_symbols_base64url(data) {
        return Err(CRT_WRONG_STRING_BASE64URL.to_string());
    }

    // Get the key from the secret string.
    let digest = hash::hash(hash::MessageDigest::md5(), secret).map_err(|err| err.to_string())?;
    let key = digest.as_ref();

    // Get the reverse array for "secret".
    let secret_rv: Vec<u8> = secret.iter().copied().rev().collect();
    // Get the 'init_vec' from the secret string.
    let digest = hash::hash(hash::MessageDigest::md5(), &secret_rv).map_err(|err| err.to_string())?;
    let iv = digest.as_ref();

    // Decodes the string into a base64 URL.
    let data2 = decode_base64url(data);
    // Decodes a base64-encoded string to bytes.
    let data3 = base64::decode_block(&data2).map_err(|err| err.to_string())?;
    let cipher = symm::Cipher::aes_128_cbc();
    // Decrypts data in one go, and returns the decrypted data.
    let decrypted = symm::decrypt(cipher, &key, Some(&iv), &data3).map_err(|e| e.to_string())?;

    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Get the private key in PEM encoded format.
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

    // ** encrypt_utf8 **
    #[test]
    fn test_encrypt_utf8_with_empty_pr_key_pem() {
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
    fn test_encrypt_utf8_with_empty_data() {
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
    fn test_encrypt_utf8_with_valid_data() {
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

    // ** decrypt_utf8 **
    #[test]
    fn test_decrypt_utf8_with_empty_pr_key_pem() {
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
    fn test_decrypt_utf8_with_empty_data() {
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
    fn test_decrypt_utf8_with_valid_data() {
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

    // ** encode_base64url **

    #[test]
    fn test_encode_base64url_with_empty_src() {
        let res = encode_base64url("");
        assert!(res.len() == 0);
    }

    #[test]
    fn test_encode_base64url_with_valid_src() {
        let res = encode_base64url("aBzSFmAGqjIr+pbLXmP3h37nM9p18IbVA/Xp0lvmw30=");
        assert_eq!(res, "aBzSFmAGqjIr-pbLXmP3h37nM9p18IbVA_Xp0lvmw30=");
    }

    // ** decode_base64url **

    #[test]
    fn test_decode_base64url_with_empty_src() {
        let res = decode_base64url("");
        assert!(res.len() == 0);
    }

    #[test]
    fn test_decode_base64url_with_valid_src() {
        let res = decode_base64url("aBzSFmAGqjIr-pbLXmP3h37nM9p18IbVA_Xp0lvmw30=");
        assert_eq!(res, "aBzSFmAGqjIr+pbLXmP3h37nM9p18IbVA/Xp0lvmw30=");
    }

    // ** encrypt_aes **

    #[test]
    fn test_encrypt_aes_with_valid_data() {
        let data = "The string is Hello World";
        let secret = "Secret phrase for encryption.";

        let res = encrypt_aes(secret.as_bytes(), data.as_bytes());
        assert!(res.is_ok());
        let r = "1yRhcTMcXiYVcKNafr98LEeNfaJ_4H1rBcj9IeYiuvE=";
        let encrypted = res.ok().unwrap();
        assert_eq!(r, &encrypted);
    }

    // ** decrypt_aes **

    #[test]
    fn test_decrypt_aes_with_empty_data() {
        let data = "";
        let secret = "Secret phrase for encryption.";

        let res = decrypt_aes(secret.as_bytes(), data);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), CRT_MESSAGE_STRING_EMPTY);
    }
    #[test]
    fn test_decrypt_aes_with_data_not_multiple_4() {
        let data = "bad";
        let secret = "Secret phrase for encryption.";

        let res = decrypt_aes(secret.as_bytes(), data);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), CRT_WRONG_STRING_BASE64URL);
    }
    #[test]
    fn test_decrypt_aes_with_invalid_data() {
        let data = "invalid!";
        let secret = "Secret phrase for encryption.";

        let res = decrypt_aes(secret.as_bytes(), data);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), CRT_WRONG_STRING_BASE64URL);
    }
    #[test]
    fn test_decrypt_aes_with_wrong_data1() {
        let data = "1y#hcTMcXiYVcKNafr98LEeNfaJ_4H1rBcj9IeYiuvE=";
        let secret = "Secret phrase for encryption.";

        let res = decrypt_aes(secret.as_bytes(), data);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), CRT_WRONG_STRING_BASE64URL);
    }

    #[test]
    fn test_decrypt_aes_with_valid_data() {
        let data = "1yRhcTMcXiYVcKNafr98LEeNfaJ_4H1rBcj9IeYiuvE=";
        let secret = "Secret phrase for encryption.";

        let res = decrypt_aes(secret.as_bytes(), data);
        assert!(res.is_ok());
        let r = "The string is Hello World".as_bytes();
        let decrypted = res.ok().unwrap();
        assert_eq!(r, &decrypted);
    }
}
