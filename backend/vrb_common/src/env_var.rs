use std::env;


pub fn env_set_var(var_name: &str, var_value: &str) {
    unsafe {
        env::set_var(var_name, var_value);
    }
}