use niri_ipc::Window;
use regex_lite::Regex;
use serde::{Deserialize, Serialize};

use crate::config;

pub fn read_rules_from_config() -> Option<Rules> {
    let rules = config::rules_path()?;
    let contents = std::fs::read_to_string(&rules).ok()?;
    toml::from_str(&contents).ok()
}

#[derive(Serialize, Deserialize)]
pub struct Rules {
    #[serde(rename = "rule")]
    inner: Vec<Rule>,
}

impl Rules {
    pub fn into_iter(self) -> impl Iterator<Item = Rule> {
        self.inner.into_iter()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Rule {
    /// Title of the window.
    title: Option<String>,
    /// AppID of the window.
    #[serde(alias = "app-id")]
    app_id: Option<String>,
}

impl Rule {
    pub fn try_compile(self) -> Result<CompiledRule, regex_lite::Error> {
        Ok(CompiledRule {
            title: self.title.map(|title| Regex::new(&title)).transpose()?,
            app_id: self.app_id,
        })
    }
}

pub struct CompiledRule {
    title: Option<Regex>,
    app_id: Option<String>,
}

impl CompiledRule {
    pub fn matches(&self, window: &Window) -> bool {
        self.title(window) && self.app_id(window)
    }

    fn title(&self, window: &Window) -> bool {
        if let Some(title_rule) = &self.title {
            window
                .title
                .as_ref()
                .is_some_and(|title| title_rule.is_match(title))
        } else {
            false
        }
    }

    fn app_id(&self, window: &Window) -> bool {
        if let Some(app_id_rule) = &self.app_id {
            window.app_id.as_ref().is_some_and(|id| app_id_rule == id)
        } else {
            false
        }
    }
}
