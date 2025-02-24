use std::{
    fs,
    io::{self, BufRead, Write},
    path,
};

use crate::utils::crypto;

const MSG_ENV_IS_NOT_SPECIFIED: &str = "The `ENV` file is not specified.";
const MSG_PKEY_IS_NOT_SPECIFIED: &str = "The private key file was not specified.";
const MSG_MAX_PRM_IS_NOT_SPECIFIED: &str = "The maximum parameter length is not specified.";

/// Checking the configuration and encrypting the specified parameters.
/// * env_file_path: `&str` - Configuration file.
/// * param_list: `&[&str]` - List of parameters to be encoded.
/// * key_file_path: `&str` - Private key file in PEM encoding format.
/// * param_len: `usize` - If the parameter length is less than the specified value, then it needs to be encoded.
///                        If it is greater, then the parameter value is already encoded.
/// * Returns: `Vec<String>` - List of parameter names that were encrypted.
///
pub fn check_params_env(
    env_file_path: &str,
    param_list: &[&str],
    key_file_path: &str,
    param_len: usize,
) -> Result<Vec<String>, String> {
    if env_file_path.len() == 0 {
        return Err(MSG_ENV_IS_NOT_SPECIFIED.to_string());
    }
    if key_file_path.len() == 0 {
        return Err(MSG_PKEY_IS_NOT_SPECIFIED.to_string());
    }
    if param_len == 0 {
        return Err(MSG_MAX_PRM_IS_NOT_SPECIFIED.to_string());
    }
    let mut vec: Vec<String> = Vec::new();

    let file_path = path::PathBuf::from(key_file_path);
    let pr_key_pem = fs::read_to_string(file_path).map_err(|err| err.to_string())?;

    // Open the file in read-only mode (ignoring errors).
    let file = fs::File::open(env_file_path).map_err(|err| err.to_string())?;
    let reader = io::BufReader::new(file);

    let mut encrypted_params: Vec<String> = Vec::new();
    if param_list.len() > 0 {
        // Read the file line by line using the lines() iterator from std::io::BufRead.
        for line in reader.lines() {
            let line = line.map_err(|err| err.to_string())?;
            if line.len() == 0 || "#".eq(&line[0..1]) {
                vec.push(line);
                continue;
            }
            let (prm_name, value) = line.split_once('=').unwrap_or(("", ""));
            let mut prm_value = value;
            if prm_name.len() == 0 || prm_value.len() == 0 {
                vec.push(line);
                continue;
            }
            let value = if param_list.contains(&prm_name) && prm_value.len() < param_len {
                encrypted_params.push(prm_name.to_string());
                crypto::encrypt_utf8(&pr_key_pem, &prm_value.as_bytes()).map_err(|err| err.to_string())?
            } else {
                prm_value.to_string()
            };
            prm_value = value.as_str();
            let txt = format!("{}={}", prm_name, prm_value);
            vec.push(txt);
        }
    }

    if encrypted_params.len() > 0 {
        // Get the name for the old file.
        let mut env_old_path = path::PathBuf::from(&env_file_path);
        env_old_path.set_extension("old");
        let env_old_name = env_old_path.to_str().unwrap();
        if path::Path::new(&env_old_name).exists() {
            let _ = fs::remove_file(&env_old_name);
        }
        // Rename the current version of the file to the old version of the file.
        fs::rename(&env_file_path, env_old_name).map_err(|err| err.to_string())?;
        // Save a new version of the file.
        let mut file = fs::File::create(&env_file_path).map_err(|err| err.to_string())?;
        for line in vec.iter() {
            file.write(&format!("{}\n", &line).as_bytes()).map_err(|err| err.to_string())?;
        }
    }

    Ok(encrypted_params)
}
/// Update configurations and decryption of specified parameters.
/// * param_list: `&[&str]` - List of parameters to be encoded.
/// * key_file_path: `&str` - Private key file in PEM encoding format.
/// * param_len: `usize` - If the parameter length is less than the specified value, then it needs to be encoded.
///                        If it is greater, then the parameter value is already encoded.
/// * Returns: `Vec<String>` - List of parameter names that were decrypted.
///
pub fn update_params_env(param_list: &[&str], key_file_path: &str, param_len: usize) -> Result<Vec<String>, String> {
    if key_file_path.len() == 0 {
        return Err(MSG_PKEY_IS_NOT_SPECIFIED.to_string());
    }
    if param_len == 0 {
        return Err(MSG_MAX_PRM_IS_NOT_SPECIFIED.to_string());
    }
    let file_path = path::PathBuf::from(key_file_path);
    let pr_key_pem = fs::read_to_string(file_path).map_err(|err| err.to_string())?;
    let mut decrypted_params: Vec<String> = Vec::new();
    for param in param_list.iter() {
        let prm_name = *param;
        let prm_value = std::env::var(prm_name).unwrap_or("".to_string());
        if prm_name.len() > 0 && prm_value.len() >= param_len {
            let value = crypto::decrypt_utf8(&pr_key_pem, &prm_value).map_err(|err| err.to_string())?;
            let value_str = std::str::from_utf8(&value).unwrap();
            std::env::set_var(prm_name, value_str);
            decrypted_params.push(prm_name.to_string());
        }
    }

    Ok(decrypted_params)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATABASE_NAME: &str = "TEST_DATABASE_URL";
    const DATABASE_VALUE: &str = "postgresql://database_user:database_password@127.0.0.1:5432/database_name";
    const HOST_PORT_NAME: &str = "TEST_SMTP_HOST_PORT";
    const HOST_PORT_VALUE: &str = "smtp.demo.com:9999";
    const USER_PASS_NAME: &str = "TEST_SMTP_USER_PASS";
    const USER_PASS_VALUE: &str = "user_demo@demo.com:user_demo";
    const SECRET_KEY_NAME: &str = "TEST_JWT_SECRET_KEY";
    const SECRET_KEY_VALUE: &str = "jwt_secret_key";

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

    // ** check_params_env **
    #[test]
    fn test_check_params_env_empty_env_file() {
        let list: Vec<&str> = Vec::new();
        let result = check_params_env(&"", &list, &"./demo01.key.pem", 500);

        assert!(result.is_err());
        assert_eq!(result.err(), Some(MSG_ENV_IS_NOT_SPECIFIED.to_string()));
    }
    #[test]
    fn test_check_params_env_empty_key_file() {
        let list: Vec<&str> = Vec::new();
        let result = check_params_env(&"./demo01_env", &list, &"", 500);

        assert!(result.is_err());
        assert_eq!(result.err(), Some(MSG_PKEY_IS_NOT_SPECIFIED.to_string()));
    }
    #[test]
    fn test_check_params_env_param_len_is_0() {
        let list: Vec<&str> = Vec::new();
        let result = check_params_env(&"./demo01_env", &list, &"./demo01.key.pem", 0);

        assert!(result.is_err());
        assert_eq!(result.err(), Some(MSG_MAX_PRM_IS_NOT_SPECIFIED.to_string()));
    }
    #[test]
    fn test_check_params_env_empty_param_list() {
        // Create private key in PEM encoded format.
        let pr_key_pem_path = "./demo11.key.pem";
        let mut file = fs::File::create(&pr_key_pem_path).unwrap();
        let pr_key_pem_str = get_pr_key_pem();
        file.write(&pr_key_pem_str.as_bytes()).unwrap();

        // Creating the "env" configuration file.
        let env_demo_path = "./demo11_env";
        let mut list: Vec<&str> = Vec::new();
        let s1 = format!("{}={}", DATABASE_NAME, DATABASE_VALUE);
        list.push(&s1);
        let s2 = format!("{}={}", HOST_PORT_NAME, HOST_PORT_VALUE);
        list.push(&s2);
        let s3 = format!("{}={}", USER_PASS_NAME, USER_PASS_VALUE);
        list.push(&s3);
        let s4 = format!("{}={}", SECRET_KEY_NAME, SECRET_KEY_VALUE);
        list.push(&s4);

        let mut file = fs::File::create(&env_demo_path).unwrap();
        for line in list.iter() {
            file.write(&format!("{}\n", &line).as_bytes()).unwrap();
        }

        // Creating a list of parameters.
        let prms: Vec<&str> = Vec::new();
        let result = check_params_env(env_demo_path, &prms, &pr_key_pem_path, 500);
        let _ = fs::remove_file(&env_demo_path);
        let _ = fs::remove_file(&pr_key_pem_path);

        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(vec![]));
    }
    #[test]
    fn test_check_params_env_valid_data() {
        // Create private key in PEM encoded format.
        let pr_key_pem_path = "./demo12.key.pem";
        let mut file = fs::File::create(&pr_key_pem_path).unwrap();
        let pr_key_pem_str = get_pr_key_pem();
        file.write(&pr_key_pem_str.as_bytes()).unwrap();

        // Creating the "env" configuration file.
        let env_demo_path = "./demo12_env";
        if path::Path::new(&env_demo_path).exists() {
            let _ = fs::remove_file(&env_demo_path);
        }
        let mut list: Vec<&str> = Vec::new();
        let s1 = format!("{}={}", DATABASE_NAME, DATABASE_VALUE);
        list.push(&s1);
        let s2 = format!("{}={}", HOST_PORT_NAME, HOST_PORT_VALUE);
        list.push(&s2);
        let s3 = format!("{}={}", USER_PASS_NAME, USER_PASS_VALUE);
        list.push(&s3);
        let s4 = format!("{}={}", SECRET_KEY_NAME, SECRET_KEY_VALUE);
        list.push(&s4);
        let mut file = fs::File::create(&env_demo_path).unwrap();
        for line in list.iter() {
            file.write(&format!("{}\n", &line).as_bytes()).unwrap();
        }

        // Creating a list of parameters.
        let mut prms: Vec<&str> = Vec::new();
        prms.push(DATABASE_NAME);
        prms.push(HOST_PORT_NAME);
        prms.push(USER_PASS_NAME);
        let prms2: Vec<String> = prms.clone().iter().map(|v| v.to_string()).collect();
        let result = check_params_env(env_demo_path, &prms, &pr_key_pem_path, 500);
        let _ = fs::remove_file(&env_demo_path);
        let _ = fs::remove_file(&pr_key_pem_path);

        let mut env_old_path = path::PathBuf::from(&env_demo_path);
        env_old_path.set_extension("old");
        let env_old_name = env_old_path.to_str().unwrap();
        let is_exist_old_file = path::Path::new(&env_old_name).exists();
        if is_exist_old_file {
            let _ = fs::remove_file(&env_old_name);
        }

        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(prms2));
        assert_eq!(is_exist_old_file, true);
    }

    // ** update_params_env **
    #[test]
    fn test_update_params_env_empty_key_file() {
        let list: Vec<&str> = Vec::new();
        let result = update_params_env(&list, &"", 500);

        assert!(result.is_err());
        assert_eq!(result.err(), Some(MSG_PKEY_IS_NOT_SPECIFIED.to_string()));
    }
    #[test]
    fn test_update_params_env_param_len_is_0() {
        let list: Vec<&str> = Vec::new();
        let result = update_params_env(&list, &"./demo02.key.pem", 0);

        assert!(result.is_err());
        assert_eq!(result.err(), Some(MSG_MAX_PRM_IS_NOT_SPECIFIED.to_string()));
    }
    #[test]
    fn test_update_params_env_empty_param_list() {
        // Create private key in PEM encoded format.
        let pr_key_pem_path = "./demo21.key.pem";
        let mut file = fs::File::create(&pr_key_pem_path).unwrap();
        let pr_key_pem_str = get_pr_key_pem();
        file.write(&pr_key_pem_str.as_bytes()).unwrap();

        // Creating the "env" configuration file.
        let env_demo_path = "./demo21_env";
        let mut list: Vec<&str> = Vec::new();
        let s1 = format!("{}={}", DATABASE_NAME, DATABASE_VALUE);
        list.push(&s1);
        let s2 = format!("{}={}", HOST_PORT_NAME, HOST_PORT_VALUE);
        list.push(&s2);
        let s3 = format!("{}={}", USER_PASS_NAME, USER_PASS_VALUE);
        list.push(&s3);
        let s4 = format!("{}={}", SECRET_KEY_NAME, SECRET_KEY_VALUE);
        list.push(&s4);

        let mut file = fs::File::create(&env_demo_path).unwrap();
        for line in list.iter() {
            file.write(&format!("{}\n", &line).as_bytes()).unwrap();
        }

        // Creating a list of parameters.
        let prms: Vec<&str> = Vec::new();
        let result = update_params_env(&prms, &pr_key_pem_path, 500);
        let _ = fs::remove_file(&env_demo_path);
        let _ = fs::remove_file(&pr_key_pem_path);

        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(vec![]));
    }
    #[test]
    fn test_update_params_env_valid_data_no_encoded() {
        // Create private key in PEM encoded format.
        let pr_key_pem_path = "./demo22.key.pem";
        let mut file = fs::File::create(&pr_key_pem_path).unwrap();
        let pr_key_pem_str = get_pr_key_pem();
        file.write(&pr_key_pem_str.as_bytes()).unwrap();

        std::env::set_var(DATABASE_NAME, DATABASE_VALUE);
        std::env::set_var(HOST_PORT_NAME, HOST_PORT_VALUE);
        std::env::set_var(USER_PASS_NAME, USER_PASS_VALUE);

        // Creating a list of parameters.
        let mut prms: Vec<&str> = Vec::new();
        prms.push(DATABASE_NAME);
        prms.push(HOST_PORT_NAME);
        prms.push(USER_PASS_NAME);

        let result = update_params_env(&prms, &pr_key_pem_path, 500);

        let _ = fs::remove_file(&pr_key_pem_path);

        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(vec![]));

        let dbase2 = std::env::var(DATABASE_NAME).unwrap_or("".to_string());
        assert_eq!(DATABASE_VALUE, dbase2);

        let host_port2 = std::env::var(HOST_PORT_NAME).unwrap_or("".to_string());
        assert_eq!(HOST_PORT_VALUE, host_port2);

        let user_pass2 = std::env::var(USER_PASS_NAME).unwrap_or("".to_string());
        assert_eq!(USER_PASS_VALUE, user_pass2);
    }
    #[test]
    fn test_update_params_env_valid_data_encoded() {
        // Create private key in PEM encoded format.
        let pr_key_pem_path = "./demo23.key.pem";
        let mut file = fs::File::create(&pr_key_pem_path).unwrap();
        let pr_key_pem_str = get_pr_key_pem();
        file.write(&pr_key_pem_str.as_bytes()).unwrap();

        let file_path = path::PathBuf::from(pr_key_pem_path);
        let pr_key_pem = fs::read_to_string(file_path).unwrap();

        let dbase = crypto::encrypt_utf8(&pr_key_pem, &DATABASE_VALUE.as_bytes()).unwrap();
        std::env::set_var(DATABASE_NAME, dbase);

        let host_port = crypto::encrypt_utf8(&pr_key_pem, &HOST_PORT_VALUE.as_bytes()).unwrap();
        std::env::set_var(HOST_PORT_NAME, host_port);

        let user_pass = crypto::encrypt_utf8(&pr_key_pem, &USER_PASS_VALUE.as_bytes()).unwrap();
        std::env::set_var(USER_PASS_NAME, user_pass);

        // Creating a list of parameters.
        let mut prms: Vec<&str> = Vec::new();
        prms.push(DATABASE_NAME);
        prms.push(HOST_PORT_NAME);
        prms.push(USER_PASS_NAME);
        let prms2: Vec<String> = prms.clone().iter().map(|v| v.to_string()).collect();

        let result = update_params_env(&prms, &pr_key_pem_path, 300);

        let _ = fs::remove_file(&pr_key_pem_path);

        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(prms2));

        let dbase2 = std::env::var(DATABASE_NAME).unwrap_or("".to_string());
        assert_eq!(DATABASE_VALUE, dbase2);

        let host_port2 = std::env::var(HOST_PORT_NAME).unwrap_or("".to_string());
        assert_eq!(HOST_PORT_VALUE, host_port2);

        let user_pass2 = std::env::var(USER_PASS_NAME).unwrap_or("".to_string());
        assert_eq!(USER_PASS_VALUE, user_pass2);
    }
}
