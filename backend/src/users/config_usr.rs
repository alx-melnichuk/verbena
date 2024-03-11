const USR_SHOW_LEAD_TIME_DEF: bool = false;

// User Properties
#[derive(Debug, Clone)]
pub struct ConfigUsr {
    // A flag to display the execution time of methods.
    pub usr_show_lead_time: bool,
}

impl ConfigUsr {
    pub fn init_by_env() -> Self {
        let def = USR_SHOW_LEAD_TIME_DEF.to_string();
        let usr_show_lead_time: bool = std::env::var("USR_SHOW_LEAD_TIME").unwrap_or(def).trim().parse().unwrap();
        ConfigUsr { usr_show_lead_time }
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub fn get_test_config() -> ConfigUsr {
    ConfigUsr {
        usr_show_lead_time: false,
    }
}
