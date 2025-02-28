use std::env;
use std::path::PathBuf;

pub fn home() -> Option<PathBuf> {
  if let Ok(home) = env::var("HOME") {
    Some(PathBuf::from(home))
  } else {
    None
  }
}

pub fn data_dir() -> Option<PathBuf> {
  //$XDG_DATA_HOME or $HOME/.local/share
  if let Ok(data_home) = env::var("XDG_DATA_HOME") {
    Some(PathBuf::from(data_home))
  } else {
    if let Some(mut data_home) = home() {
      data_home.push(".local");
      data_home.push("share");
      Some(data_home)
    } else {
      None
    }
  }
}

pub fn config_dir() -> Option<PathBuf> {
  //$XDG_CONFIG_HOME or $HOME/.config
  if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
    Some(PathBuf::from(config_home))
  } else {
    if let Some(mut config_home) = home() {
      config_home.push(".config");
      Some(config_home)
    } else {
      None
    }
  }
}

