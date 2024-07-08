use directories::ProjectDirs;
use std::{
    fs::{self,File},
    io::{Read, Write},
    path::PathBuf,
    sync::OnceLock,
};

pub const APP_QUALIFIER: &str = "com";
pub const APP_ORGANIZATION: &str = "moxin-org";
pub const APP_NAME: &str = "moxin";

pub fn project_dirs() -> &'static ProjectDirs {
    // This can be redesigned once std::sync::LazyLock is stabilized.
    static MOXIN_PROJECT_DIRS: OnceLock<ProjectDirs> = OnceLock::new();

    MOXIN_PROJECT_DIRS.get_or_init(|| {
        ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
            .expect("Failed to obtain Moxin project directories")
    })
}

pub const MODEL_DOWNLOADS_DIR_NAME: &str = "model_downloads";
pub fn setup_model_downloads_folder() -> PathBuf {
    let downloads_dir = project_dirs().data_dir().join(MODEL_DOWNLOADS_DIR_NAME);

    std::fs::create_dir_all(&downloads_dir).unwrap_or_else(|_| {
        panic!(
            "Failed to create the model downloads directory at {:?}",
            downloads_dir
        )
    });
    downloads_dir
}

pub const CHATS_DIR_NAME: &str = "chats";
pub fn setup_chats_folder() -> PathBuf {
    let chats_dir = project_dirs().data_dir().join(CHATS_DIR_NAME);

    std::fs::create_dir_all(&chats_dir)
        .unwrap_or_else(|_| panic!("Failed to create the chats directory at {:?}", chats_dir));
    chats_dir
}

pub fn setup_preferences_folder() -> PathBuf {
    let preferences_dir = project_dirs().preference_dir();

    std::fs::create_dir_all(&preferences_dir).unwrap_or_else(|_| {
        panic!(
            "Failed to create the preferences directory at {:?}",
            preferences_dir
        )
    });
    preferences_dir.to_path_buf()
}

pub fn read_from_file(path: PathBuf) -> Result<String, std::io::Error> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(why) => return Err(why),
    };

    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => Ok(json),
        Err(why) => Err(why),
    }
}

pub fn write_to_file(path: PathBuf, json: &str) -> Result<(), std::io::Error> {
    // Ensure the directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create or overwrite the file
    let mut file = File::create(path)?;

    // Write the JSON data to the file
    file.write_all(json.as_bytes())?;
    Ok(())
}
