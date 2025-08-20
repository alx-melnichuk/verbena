use std::env;

use crate::alias_path::alias_path::AliasPath;
use crate::consts;

#[derive(Debug, Clone)]
pub struct AliasStrm {
    adapter_alias: AliasPath,
}

impl AliasStrm {
    pub fn new() -> Self {
        let logo_files_dir = env::var(consts::STRM_LOGO_FILES_DIR).unwrap_or(consts::LOGO_FILES_DIR.to_string());

        let adapter_alias = AliasPath::new(consts::ALIAS_LOGO_FILES_DIR, &logo_files_dir);
        AliasStrm { adapter_alias }
    }
    pub fn path_to_alias(&self, full_path_file: &str) -> String {
        self.adapter_alias.path_to_alias(full_path_file)
    }
    pub fn alias_to_path(&self, full_path_file: &str) -> String {
        self.adapter_alias.alias_to_path(full_path_file)
    }
}
