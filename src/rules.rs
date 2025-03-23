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
        let conditions = [
            self.title(window),
            self.app_id(window),
        ];

        satisfied(conditions)
    }

    fn title(&self, window: &Window) -> Option<bool> {
        self.title
            .as_ref()
            .and_then(|title_rule| {
                window
                    .title
                    .as_ref()
                    .map(|title| title_rule.is_match(title))
            })
    }

    fn app_id(&self, window: &Window) -> Option<bool> {
        self.app_id
            .as_ref()
            .and_then(|app_id_rule| window.app_id.as_ref().map(|id| app_id_rule == id))
    }
}

/// Only returns `true` when all values are `Some(true)`` or `None``.
fn satisfied(iter: impl IntoIterator<Item = Option<bool>>) -> bool {
    iter.into_iter().fold(false, |acc, tri| {
        match tri {
            Some(true) => acc | true,
            Some(false) => acc & false,
            None => acc | false,
        }
    })
}

#[test]
fn satisfied_all_cases() {
    assert!(satisfied([Some(true), None, None]));
    assert!(satisfied([Some(true), Some(true), None]));
    assert!(satisfied([Some(true), Some(true), Some(true)]));
    
    assert!(!satisfied([]));
    assert!(!satisfied([None]));
    assert!(!satisfied([None, None, None]));
    assert!(!satisfied([Some(false), None, None]));
    assert!(!satisfied([Some(false), Some(false), None]));
    assert!(!satisfied([Some(true), Some(false), None]));
}