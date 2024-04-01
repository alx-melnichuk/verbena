use std::{
    fs,
    io::{self, BufRead, Write},
    path,
};

use crate::utils::crypto;

// check("./.env", &list, "./example.key.pem", 500);
pub fn check_params_env(
    env_file_path: &str,
    params: &[&str],
    key_file_path: &str,
    param_len: usize,
) -> Result<(), String> {
    if env_file_path.len() == 0 {
        return Err("The `ENV` file is not specified.".to_string());
    }
    if params.len() == 0 {
        return Err("Parameters not specified".to_string());
    }
    if key_file_path.len() == 0 {
        return Err("The private key file was not specified.".to_string());
    }
    if param_len == 0 {
        return Err("The maximum parameter length is not specified.".to_string());
    }
    let file_path = path::PathBuf::from(key_file_path);
    let pr_key_pem = fs::read_to_string(file_path).map_err(|err| err.to_string())?;

    // Open the file in read-only mode (ignoring errors).
    let file = fs::File::open(env_file_path).map_err(|err| err.to_string())?;
    let reader = io::BufReader::new(file);
    let mut vec: Vec<String> = Vec::new();

    let mut amount_crypto = 0;
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
        let value = if params.contains(&prm_name) && prm_value.len() < param_len {
            amount_crypto = amount_crypto + 1;
            crypto::encrypt_utf8(&pr_key_pem, &prm_value.as_bytes()).map_err(|err| err.to_string())?
        } else {
            prm_value.to_string()
        };
        prm_value = value.as_str();
        let txt = format!("{}={}", prm_name, prm_value);
        vec.push(txt);
    }

    if amount_crypto > 0 {
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

    Ok(())
}

pub fn update_params_env(params: &[&str], key_file_path: &str, param_len: usize) -> Result<(), String> {
    if params.len() == 0 {
        return Err("Parameters not specified".to_string());
    }
    if key_file_path.len() == 0 {
        return Err("The private key file was not specified.".to_string());
    }
    if param_len == 0 {
        return Err("The maximum parameter length is not specified.".to_string());
    }
    let file_path = path::PathBuf::from(key_file_path);
    let pr_key_pem = fs::read_to_string(file_path).map_err(|err| err.to_string())?;

    for param in params.iter() {
        let prm_name = *param;
        let prm_value = std::env::var(prm_name).unwrap_or("".to_string());
        if prm_name.len() > 0 && prm_value.len() >= param_len {
            let value = crypto::decrypt_utf8(&pr_key_pem, &prm_value).map_err(|err| err.to_string())?;
            let value_str = std::str::from_utf8(&value).unwrap();
            std::env::set_var(prm_name, value_str);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo1() {
        eprintln!("test_demo1()");
        let mut list: Vec<&str> = Vec::new();
        list.push("DATABASE_URL");
        list.push("SMTP_HOST_PORT");
        list.push("SMTP_USER_PASS");

        let result = check_params_env("./.env", &list, "./example.key.pem", 500);
        eprintln!("result: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_demo2() {
        eprintln!("test_demo2()");
        let mut list: Vec<&str> = Vec::new();
        list.push("DATABASE_URL");
        list.push("SMTP_HOST_PORT");
        list.push("SMTP_USER_PASS");

        let result = update_params_env(&list, "./example.key.pem", 500);
        eprintln!("result: {:?}", result);
        assert!(result.is_ok());
    }
}
