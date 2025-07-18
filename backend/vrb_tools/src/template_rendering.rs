use std::{collections::HashMap, path::Path};

use handlebars::Handlebars;

pub fn render_template<P>(tpl_vec: &[(&str, P)], params: HashMap<&str, &str>) -> Result<String, String>
    where P: AsRef<Path>,
{
    if tpl_vec.len() == 0 {
        return Err("The parameter 'tpl_vec' is empty.".to_string());
    }

    let mut handlebars = Handlebars::new();
    let mut name = "";
    
    for (tpl_name, tpl_path1) in tpl_vec.into_iter() {
        let tpl_path = (*tpl_path1).as_ref();
        if tpl_name.len() == 0 || tpl_path.to_string_lossy().len() == 0 {
            continue;
        }
        if name.len() == 0 {
            name = (*tpl_name).as_ref();
        }
        handlebars.register_template_file(*tpl_name, tpl_path).map_err(|e| e.to_string())?;
    }

    if name.len() == 0 {
        return Err("The template name is not defined.".to_string());
    }

    let mut data = serde_json::json!({});
    for (key, value) in params {
        data[key] = serde_json::Value::String(value.to_string());
    }
    let content_template = handlebars.render(name, &data).map_err(|e| e.to_string())?;

    Ok(content_template)
}
