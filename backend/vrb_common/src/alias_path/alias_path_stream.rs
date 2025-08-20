use crate::alias_path::alias_path::AliasPath;
use crate::consts;

#[derive(Debug, Clone)]
pub struct AliasStrm {
    adapter_alias: AliasPath,
}

impl AliasStrm {
    // env::var(consts::STRM_LOGO_FILES_DIR).unwrap_or(consts::LOGO_FILES_DIR.to_string())
    pub fn new(path: &str) -> Self {
        AliasStrm { adapter_alias: AliasPath::new(consts::ALIAS_LOGO_FILES_DIR, path) }
    }
    /// Replace file path prefix with alias.
    pub fn path_to_alias(&self, full_path_file: &str) -> String {
        self.adapter_alias.path_to_alias(full_path_file)
    }
    /// Return file path prefix instead of alias.
    pub fn alias_to_path(&self, full_path_file: &str) -> String {
        self.adapter_alias.alias_to_path(full_path_file)
    }
}
