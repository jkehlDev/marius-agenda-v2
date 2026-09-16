use crate::config::AgendaConfig;
use crate::create_default_config;
use crate::paths::{config_path, resolve_data_dir};
use std::fs;
use std::io;
use std::path::Path;

pub fn load_config(root: &Path) -> io::Result<AgendaConfig> {
    let data_dir = resolve_data_dir(root);
    let file = config_path(&data_dir);
    match fs::read_to_string(&file) {
        Ok(raw) => {
            let config: AgendaConfig = serde_json::from_str(&raw)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(config)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let defaults = create_default_config();
            save_config(root, &defaults)?;
            Ok(defaults)
        }
        Err(e) => Err(e),
    }
}

pub fn save_config(root: &Path, config: &AgendaConfig) -> io::Result<()> {
    save_config_to(root, &resolve_data_dir(root), config)
}

pub fn save_config_to(_root: &Path, data_dir: &Path, config: &AgendaConfig) -> io::Result<()> {
    fs::create_dir_all(data_dir)?;
    fs::create_dir_all(data_dir.join("images"))?;
    fs::create_dir_all(data_dir.join("images/print"))?;
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(config_path(data_dir), json)?;
    Ok(())
}

pub fn load_config_from(data_dir: &Path) -> io::Result<AgendaConfig> {
    let file = config_path(data_dir);
    let raw = fs::read_to_string(&file)?;
    serde_json::from_str(&raw).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
