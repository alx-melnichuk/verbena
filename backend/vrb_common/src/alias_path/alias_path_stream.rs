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
}

impl AsRef<AliasPath> for AliasStrm {
    fn as_ref(&self) -> &AliasPath {
        &self.adapter_alias
    }
}