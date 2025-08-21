use crate::alias_path::alias_path::AliasPath;
use crate::consts;

#[derive(Debug, Clone)]
pub struct AliasPrfl {
    adapter_alias: AliasPath,
}

impl AliasPrfl {
    // env::var(consts::PRFL_AVATAR_FILES_DIR).unwrap_or(consts::AVATAR_FILES_DIR.to_string())
    pub fn new(path: &str) -> Self {
        AliasPrfl { adapter_alias: AliasPath::new(consts::ALIAS_AVATAR_FILES_DIR, path) }
    }
}

impl AsRef<AliasPath> for AliasPrfl {
    fn as_ref(&self) -> &AliasPath {
        &self.adapter_alias
    }
}