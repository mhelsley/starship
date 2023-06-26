use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(schemars::JsonSchema),
    schemars(deny_unknown_fields)
)]
#[serde(default)]
pub struct TerminalConfig<'a> {
    pub title_format: &'a str,
    pub disabled: bool,
}

impl<'a> Default for TerminalConfig<'a> {
    fn default() -> Self {
        TerminalConfig {
            title_format: "$username@$hostname$localip:$directory",
            disabled: false,
        }
    }
}
