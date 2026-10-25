use crate::{APP_NAME, APP_ORGANIZATION, APP_QUALIFIER};
use directories::ProjectDirs;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub struct Storage {
    config_dir: PathBuf,
    state_dir: PathBuf,
}

impl Storage {
    pub fn new() -> Self {
        let project_dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
            .expect("Failed to determine plateform paths");
        let config_dir = project_dirs.config_dir().to_path_buf();
        let state_dir = project_dirs.data_local_dir().to_path_buf();
        if let Err(err) = fs::create_dir_all(&config_dir) {
            tracing::error!(
                path = ?config_dir,
                %err,
                "Failed to create configuration directory"
            );
        };
        if let Err(err) = fs::create_dir_all(&state_dir) {
            tracing::error!(
                path = ?config_dir,
                %err,
                "Failed to create state directory"
            );
        };

        Self {
            config_dir,
            state_dir,
        }
    }
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }
    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }
    pub fn settings_file(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }
    pub fn keybindings_file(&self) -> PathBuf {
        self.config_dir.join("keybindings.json")
    }
    pub fn session_file(&self) -> PathBuf {
        self.state_dir.join("session.json")
    }
}

pub fn load_json<T: DeserializeOwned + Default>(path: &Path) -> T {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|err| {
            tracing::error!(
                ?path,
                %err,
                "Failed to deserialize file"
            );
            T::default()
        }),
        Err(err) if err.kind() == io::ErrorKind::NotFound => T::default(),
        Err(err) => {
            tracing::error!(
                ?path,
                %err,
                "Failed to open file"
            );
            T::default()
        }
    }
}

pub fn to_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).expect("json should always be serializable")
}

pub fn write_atomic(path: &Path, contents: &str) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, contents)?;
    std::fs::rename(&tmp, path)
}
