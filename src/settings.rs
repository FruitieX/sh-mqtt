use std::collections::HashMap;

use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct MqttSettings {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Rule {
    /// Value to compare published message against.
    pub matching_value: Option<serde_json::Value>,

    /// Check published message value at the specified JSON pointer against
    /// `matching_value`.
    pub ptr: Option<jsonptr::PointerBuf>,

    /// Ignore message if value remains unchanged. Defaults to true.
    pub ignore_unchanged: Option<bool>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct CmdSettings {
    /// Rule that must match for the command to be executed.
    pub rule: Rule,

    /// Command to be executed.
    pub cmd: String,

    /// Arguments to be passed to the command.
    pub args: Vec<String>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct TopicSettings {
    pub topic: String,
    pub commands: HashMap<String, CmdSettings>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Settings {
    pub mqtt: MqttSettings,
    pub topics: HashMap<String, TopicSettings>,
}

pub fn read_settings() -> Result<Settings, config::ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("Settings"))
        .build()?
        .try_deserialize::<Settings>()
}
