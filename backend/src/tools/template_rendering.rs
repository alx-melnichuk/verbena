use std::collections::HashMap;

use handlebars::Handlebars;

pub fn render_template(template: &str, params: HashMap<&str, &str>) -> Result<String, String> {
    if template.len() == 0 {
        return Err("The template name is not defined.".to_string());
    }
    let template_path = format!("./templates/{}.hbs", template);
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_file(template, &template_path)
        .map_err(|e| e.to_string())?;

    handlebars
        .register_template_file("base", "./templates/basic_layout.hbs")
        .map_err(|e| e.to_string())?;

    let mut data = serde_json::json!({});
    for (key, value) in params {
        data[key] = serde_json::Value::String(value.to_string());
    }

    let content_template = handlebars.render(template, &data).map_err(|e| e.to_string())?;

    Ok(content_template)
}
