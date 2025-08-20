use std::env;

use crate::alias_path::alias_path::AliasPath;

pub const ALIAS_LOGO_FILES_DIR: &str = "/logo";
pub const LOGO_FILES_DIR: &str = "./imgs/logo";

#[derive(Debug, Clone)]
pub struct AliasStrm {
    adapter_alias: AliasPath,
}

impl AliasStrm {
    pub fn new() -> Self {
        let logo_files_dir = env::var("STRM_LOGO_FILES_DIR").unwrap_or(LOGO_FILES_DIR.to_string());

        let adapter_alias = AliasPath::new(ALIAS_LOGO_FILES_DIR, &logo_files_dir);
        AliasStrm { adapter_alias }
    }
    pub fn path_to_alias(&self, full_path_file: &str) -> String {
        self.adapter_alias.path_to_alias(full_path_file)
    }
    pub fn alias_to_path(&self, full_path_file: &str) -> String {
        self.adapter_alias.alias_to_path(full_path_file)
    }
}
