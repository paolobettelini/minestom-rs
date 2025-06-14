use serde::Deserialize;
use std::{fs, path::Path};

pub fn parse_toml_config<P: AsRef<Path>, ConfigType: for<'a> Deserialize<'a>>(
    config_path: P,
) -> anyhow::Result<Box<ConfigType>> {
    let content = fs::read_to_string(config_path)?;

    let config = toml::from_str(&content)?;

    Ok(Box::new(config))
}
