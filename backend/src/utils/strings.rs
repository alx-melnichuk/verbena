pub fn msg(template: &str, params: &[&str]) -> String {
    let list: Vec<&str> = template.split("{}").collect();
    let mut result: String = String::new();

    let mut index: usize = 0;
    for part in list.iter() {
        result += part;
        let value = params.get(index).unwrap_or(&"");
        result = format!("{}{}", result, *value);
        index += 1;
    }
    result
}
