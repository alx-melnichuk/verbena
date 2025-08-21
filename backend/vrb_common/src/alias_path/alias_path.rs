use std::path::Path;

use crate::file_path;

#[derive(Debug, Clone)]
pub struct AliasPath {
    alias: String,
    path: String,
}

impl AliasPath {
    pub fn new(alias: &str, path: &str) -> Self {
        let path1 = file_path::path_complete(alias);
        let path2 = file_path::path_complete(path);
        AliasPath { alias: path1, path: path2 }
    }
    /// Replace file path prefix with alias.
    pub fn path_to_alias(&self, full_path_file: &str) -> String {
        let path = Path::new(full_path_file);
        if path.starts_with(&self.path) {
            let new_path = Path::new(&self.alias);
            let file_name = path.strip_prefix(&self.path).unwrap();
            new_path.join(file_name).to_str().unwrap().to_string()
        } else {
            full_path_file.into()
        }
    }
    /// Return file path prefix instead of alias.
    pub fn alias_to_path(&self, full_path_file: &str) -> String {
        let path = Path::new(full_path_file);
        if path.starts_with(&self.alias) {
            let new_path = Path::new(&self.path);
            let file_name = path.strip_prefix(&self.alias).unwrap();
            new_path.join(file_name).to_str().unwrap().to_string()
        } else {
            full_path_file.into()
        }
    }
    /// Returns true if the string prefix matches the value of the alias.
    pub fn starts_with_alias(&self, full_path_file: &str) -> bool {
        full_path_file.starts_with(&self.alias)
    }
}

#[cfg(test)]
mod tests {
    use crate::alias_path::alias_path::AliasPath;

    #[actix_web::test]
    async fn test_alias_path_single_alias_single_path() {
        let alias = "alias1".to_string();
        let path = "path1".to_string();
    
        let adp = AliasPath::new(&alias, &path);
        
        let file_name1 = "file_name1.txt".to_string();

        let res1 = adp.path_to_alias(&format!("{}/{}", &path, &file_name1));
        assert_eq!(res1, format!("{}/{}", alias, file_name1));

        let res1 = adp.alias_to_path(&format!("{}/{}", &alias, &file_name1));
        assert_eq!(res1, format!("{}/{}", path, file_name1));
    }
    #[actix_web::test]
    async fn test_alias_path_single_alias_without_path() {
        let alias = "alias1".to_string();
        let path = "path1".to_string();
    
        let adp = AliasPath::new(&alias, &path);
        
        let file_name1 = "file_name1.txt".to_string();

        let path_file1 = format!("test{}/{}", &path, &file_name1);
        let res1 = adp.path_to_alias(&path_file1);
        assert_eq!(res1, path_file1);

        let res1 = adp.alias_to_path(&path_file1);
        assert_eq!(res1, path_file1);
    }
    #[actix_web::test]
    async fn test_alias_path_simple_alias_simple_path() {
        let alias = "/alias1".to_string();
        let path = "/path1".to_string();
    
        let adp = AliasPath::new(&alias, &path);
        
        let file_name1 = "file_name1.txt".to_string();

        let res1 = adp.path_to_alias(&format!("{}/{}", &path, &file_name1));
        assert_eq!(res1, format!("{}/{}", alias, file_name1));

        let res1 = adp.alias_to_path(&format!("{}/{}", &alias, &file_name1));
        assert_eq!(res1, format!("{}/{}", path, file_name1));
    }
    #[actix_web::test]
    async fn test_alias_path_complex_alias_complex_path() {
        let alias = "/alias1/test1".to_string();
        let path = "/path1/demo1".to_string();
    
        let adp = AliasPath::new(&alias, &path);
        
        let file_name1 = "file_name1.txt".to_string();

        let res1 = adp.path_to_alias(&format!("{}/{}", &path, &file_name1));
        assert_eq!(res1, format!("{}/{}", alias, file_name1));

        let res1 = adp.alias_to_path(&format!("{}/{}", &alias, &file_name1));
        assert_eq!(res1, format!("{}/{}", path, file_name1));
    }
    #[actix_web::test]
    async fn test_alias_path_complex_alias_dir_complex_path_dir() {
        let alias = "/alias1/test1/".to_string();
        let path = "/path1/demo1/".to_string();
    
        let adp = AliasPath::new(&alias, &path);
        
        let file_name1 = "file_name1.txt".to_string();

        let res1 = adp.path_to_alias(&format!("{}{}", &path, &file_name1));
        assert_eq!(res1, format!("{}{}", alias, file_name1));

        let res1 = adp.alias_to_path(&format!("{}{}", &alias, &file_name1));
        assert_eq!(res1, format!("{}{}", path, file_name1));
    }
    #[actix_web::test]
    async fn test_alias_path_simple_alias_dir_complex_path_dir() {
        let alias = "/alias1/".to_string();
        let path = "/path1/demo1/".to_string();
    
        let adp = AliasPath::new(&alias, &path);
        
        let file_name1 = "file_name1.txt".to_string();

        let res1 = adp.path_to_alias(&format!("{}{}", &path, &file_name1));
        assert_eq!(res1, format!("{}{}", alias, file_name1));

        let res1 = adp.alias_to_path(&format!("{}{}", &alias, &file_name1));
        assert_eq!(res1, format!("{}{}", path, file_name1));
    }
    #[actix_web::test]
    async fn test_alias_path_simple_alias_dir_complex_path_dir_file_dir() {
        let alias = "/alias1/".to_string();
        let path = "/path1/demo1/".to_string();
    
        let adp = AliasPath::new(&alias, &path);
        
        let file_name1 = "test/file_name1.txt".to_string();

        let res1 = adp.path_to_alias(&format!("{}{}", &path, &file_name1));
        assert_eq!(res1, format!("{}{}", alias, file_name1));

        let res1 = adp.alias_to_path(&format!("{}{}", &alias, &file_name1));
        assert_eq!(res1, format!("{}{}", path, file_name1));
    }
}