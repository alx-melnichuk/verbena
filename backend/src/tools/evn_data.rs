use std::{
    fs,
    io::{self, BufRead},
    path,
};

use crate::utils::crypto;

pub fn check(env_file_path: &str, params: &[&str]) -> Result<String, String> {
    eprintln!("check()");

    let file_path = path::PathBuf::from("./example.key.pem");
    let pr_key_pem = fs::read_to_string(file_path).unwrap();
    eprintln!("pr_key_pem: {}", pr_key_pem);
    eprintln!("pr_key_pem.len(): {}", pr_key_pem.len());

    // Open the file in read-only mode (ignoring errors).
    let file = fs::File::open(env_file_path).map_err(|err| err.to_string())?;
    let reader = io::BufReader::new(file);
    let mut vec: Vec<String> = Vec::new();

    // Read the file line by line using the lines() iterator from std::io::BufRead.
    for line in reader.lines() {
        let line = line.map_err(|err| err.to_string())?;
        eprint!("{}", &line);
        if line.len() == 0 || "#".eq(&line[0..1]) {
            eprintln!("");
            vec.push(line);
            continue;
        }
        let parts: Vec<&str> = line.split('=').collect();
        let prm_name = parts.get(0).map(|v| v.as_ref()).unwrap_or("");
        let mut prm_value = parts.get(1).map(|v| v.as_ref()).unwrap_or("");
        eprint!("     p_name: `{}` p_val: `{}`", prm_name, prm_value);
        if prm_name.len() == 0 || prm_value.len() == 0 {
            eprintln!("");
            vec.push(line);
            continue;
        }
        let value = if params.contains(&prm_name) {
            crypto::encrypt_utf8(&pr_key_pem, &prm_value.as_bytes()).map_err(|err| err.to_string())?
        } else {
            String::new()
        };
        prm_value = value.as_str();
        eprintln!("");
        let txt = format!("{}={}", prm_name, prm_value);
        eprintln!("!! {}", &txt);
        vec.push(txt);
    }

    // let file_path = path::PathBuf::from(env_file_path);
    // let env_data = fs::read_to_string(file_path).unwrap();

    // eprintln!("env_data: {}", env_data);
    // eprintln!("env_data.len(): {}", env_data.len());

    Ok("".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo1() {
        eprintln!("test_demo1()");
        let mut list: Vec<&str> = Vec::new();
        list.push("STRM_LOGO_MAX_WIDTH");
        list.push("STRM_LOGO_MAX_HEIGHT");

        let result = check("./.env", &list);
        assert!(result.is_ok());
    }
}
