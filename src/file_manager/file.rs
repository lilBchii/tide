use dirs_next::config_dir;
use rfd::FileDialog;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use typst::syntax::FileId;

use crate::editor::buffer::Buffer;

/// Maximum number of recent projects to cache.
const MAX_CACHED_PROJECTS: usize = 5;

/// Opens a "Save File" dialog (using `RFD`) filtered by the given file type and extensions.
///
/// Returns the selected [`PathBuf`] or `None` if the dialog was canceled.
pub fn save_file_dialog(
    filter_name: &str,
    extensions: &[&str],
) -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Export Project")
        .add_filter(filter_name, extensions)
        .save_file()
}

/// Opens a "Load File" dialog (using `RFD`) filtered by file type and extension.
///
/// Returns the selected [`PathBuf`] or `None` if canceled.
pub fn load_file_dialog(
    dir: &PathBuf,
    filter_name: &str,
    extension: &[&str],
) -> Option<PathBuf> {
    FileDialog::new()
        .set_directory(dir)
        .set_title("Load File")
        .add_filter(filter_name, extension)
        .pick_file()
}

/// Opens a folder selection dialog (using `RFD`) for loading a project repository.
///
/// Returns the selected folder path, or `None` if canceled.
pub fn load_repo_dialog() -> Option<PathBuf> {
    FileDialog::new().set_title("Open Project").pick_folder()
}

/// Converts an absolute path into a path relative to the current directory/project.
///
/// Prefixes the result with `/` to maintain Typst virtual path format.
///
/// Returns `None` if the path is not relative to `current_dir`.
pub fn get_relative_path(
    current_dir: &PathBuf,
    file_path: &PathBuf,
) -> Option<PathBuf> {
    file_path.strip_prefix(current_dir).ok().map(|relative| {
        let mut result = PathBuf::from("/");
        result.push(relative);
        result
    })
}

/// Retrieves the base environment path for Tide inside the system config directory.
///
/// Creates the directory (and required subdirectories) if it does not exist.
/// Logs errors and directory creation status accordingly.
///
/// Returns the full path to the Tide config directory or `None` if creation failed.
fn retrieve_env_path() -> Option<PathBuf> {
    let path = config_dir()?.join("Tide");

    if path.exists() {
        if !path.is_dir() {
            eprintln!("Path exists but is not a directory");
        } else {
            return Some(path);
        }
    } else {
        println!("Env not set yet!");
        if let Err(e) = fs::create_dir_all(&path) {
            eprintln!("Error when creating Tide directory: {}", e);
        }
        let templates_path = path.join("templates");
        let fonts_path = path.join("fonts");
        if let Err(e) = fs::create_dir_all(&templates_path)
            .and_then(|_| fs::create_dir_all(&fonts_path))
        {
            eprintln!("Error when creating templates directory: {}", e);
        } else {
            println!("Tide env, templates and font repositories created!");
            return Some(path);
        }
    }

    None
}

/// Returns the path to the `templates` directory inside the Tide config environment.
///
/// Returns `None` if the environment or `templates` folder is missing or invalid.
pub fn get_templates_path() -> Option<PathBuf> {
    let env_path = retrieve_env_path()?;
    let templates_path = env_path.join("templates");
    if templates_path.exists() && templates_path.is_dir() {
        return Some(templates_path);
    }

    None //something went wrong here because retrieve_env_path() only returns None when creating directories failed
}

/// Returns the path to the `fonts` directory inside the Tide config environment.
///
/// Returns `None` if the environment or `fonts` folder is missing or invalid.
pub fn get_fonts_path() -> Option<PathBuf> {
    let env_path = retrieve_env_path()?;
    let fonts_path = env_path.join("fonts");
    if fonts_path.exists() && fonts_path.is_dir() {
        return Some(fonts_path);
    }

    None //something went wrong here because retrieve_env_path() only returns None when creating directories failed
}

/// Returns the path to the user-defined `config.toml` file inside the Tide environment.
///
/// This function does *not* create the file, it must be user-provided.
///
/// Returns `None` if the file does not exist or is not a regular file.
pub fn get_config_path() -> Option<PathBuf> {
    let env_path = retrieve_env_path()?;
    let config_path = env_path.join("config.toml");
    if config_path.exists() && config_path.is_file() {
        return Some(config_path);
    }

    None //we DON'T create a config.toml file in the env, the default is set instead! It is up to the user to create it
}

/// Loads the list of recently used project paths from the cache file.
///
/// Each line in the cache is expected in the format: `root_path,main_path`,
/// where `main_path` may be `?` if undefined.
///
/// Returns a list of [`ProjectCache`] instances.
pub fn get_recent_paths() -> Vec<ProjectCache> {
    let mut paths = vec![];
    if let Some(env_path) = retrieve_env_path() {
        let cache = env_path.join("recent.cache");
        match fs::read_to_string(&cache) {
            Ok(contents) => {
                for line in contents.lines() {
                    let elements: Vec<&str> = line.split(",").collect();
                    let root_path = PathBuf::from(elements[0]);
                    let main = if elements.len() < 2 || elements[1] == "?" {
                        None
                    } else {
                        Some(PathBuf::from(elements[1]))
                    };
                    paths.push(ProjectCache::new(root_path, main));
                }
            }
            Err(e) => {
                println!("Error reading cache: {}", e);
            }
        }
    }

    paths
}

/// Adds a project to the recent projects cache, ensuring no duplicates.
///
/// Keeps only the `MAX_CACHED_PROJECTS` most recent entries.
///
/// The cache is persisted to disk inside the Tide environment directory.
pub async fn cache_project(project: ProjectCache) {
    let mut projects = get_recent_paths();
    if projects.len() > MAX_CACHED_PROJECTS - 1 {
        projects.pop();
    } //we keep the last 5 projects
    projects.retain(|p| *p.root_path != project.root_path); //remove the project if it's already in cache
    projects.insert(0, project); //add to cache

    if let Some(env_path) = retrieve_env_path() {
        let cache = env_path.join("recent.cache");
        if !cache.exists() {
            match fs::File::create(&cache) {
                Ok(_) => {
                    println!("Cache file created as it didn't exist");
                }
                Err(e) => {
                    println!("Can't create cache file: {e}, abort");
                    return;
                }
            }
        }
        if cache.exists() && cache.is_file() {
            match fs::write(
                &cache,
                projects
                    .iter()
                    .map(|p| {
                        format![
                            "{},{}",
                            p.root_path.display().to_string(),
                            p.main
                                .to_owned()
                                .unwrap_or(PathBuf::from("?"))
                                .display()
                                .to_string()
                        ]
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
            ) {
                Ok(_) => {
                    println!("Project cached into recent files!");
                }
                Err(e) => {
                    println!("Error writing cache: {}", e);
                }
            }
        }
    }
}

/// Saves the given in-memory [`Buffer`] to disk at the specified directory location.
///
/// Reconstructs the full path using the file's virtual path.
///
/// # Errors
///
/// Returns an [`ErrorKind`] if writing to disk fails.
pub async fn save_file_disk(
    id: FileId,
    file: Buffer,
    dir_path: PathBuf,
) -> Result<PathBuf, ErrorKind> {
    let path = dir_path.join(id.vpath().as_rootless_path());
    fs::write(&path, file.content.text()).map_err(|e| e.kind())?;
    Ok(path)
}

/// Deletes the file corresponding to the given ID from disk.
///
/// The full path is constructed from the virtual path and the given directory (project) root.
pub fn delete_file_from_disk(
    id: FileId,
    dir_path: PathBuf,
) -> Result<(), std::io::Error> {
    let path = dir_path.join(id.vpath().as_rootless_path());
    fs::remove_file(path)?;
    Ok(())
}

/// Represents a cached project used for quick access from the recent files list.
pub struct ProjectCache {
    /// Absolute path to the root of the project.
    pub root_path: PathBuf,
    /// Optional path to the project's main Typst file.
    pub main: Option<PathBuf>,
}

impl ProjectCache {
    /// Creates a new [`ProjectCache`] from a root path and optional main file.
    pub fn new(
        root_path: PathBuf,
        main: Option<PathBuf>,
    ) -> Self {
        Self { root_path, main }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::text_editor::Content;
    use std::fmt::Debug;
    use std::ops::Add;
    use typst::syntax::VirtualPath;

    macro_rules! fake_user {
        ($s:literal) => {
            PathBuf::from(concat!("/home/alice/test/", $s))
        };
    }

    const TEST_FILE_NAME: &str = "test_save.typ";
    const TEST_CONTENT: &str = "this is a test";
    const TEST_PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");

    impl PartialEq<Self> for ProjectCache {
        fn eq(
            &self,
            other: &Self,
        ) -> bool {
            self.root_path.eq(&other.root_path) && self.main.eq(&other.main)
        }
    }

    impl Eq for ProjectCache {}

    impl Debug for ProjectCache {
        fn fmt(
            &self,
            f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            write!(
                f,
                "ProjectCache {{ root_path: {:?}, main: {:?} }}",
                self.root_path, self.main
            )
        }
    }

    impl Clone for ProjectCache {
        fn clone(&self) -> Self {
            Self {
                root_path: self.root_path.to_path_buf(),
                main: self.main.to_owned(),
            }
        }
    }

    fn check_cached_projects(
        pos: usize,
        expected: &ProjectCache,
    ) -> bool {
        let cache = get_recent_paths();
        if let Some(cached) = cache.get(pos) {
            println!("{:?}/{:?}", cached, expected);
            return cached == expected;
        }
        false
    }

    fn check_file_saved(
        file_name: &str,
        expected_content: String,
    ) -> bool {
        let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(file_name);
        let content = fs::read_to_string(&file_path).unwrap_or(String::new());

        file_path.exists() && file_path.is_file() && content.eq(&expected_content)
    }

    async fn create_test_file() -> Result<PathBuf, ErrorKind> {
        let file_id = FileId::new(None, VirtualPath::new(TEST_FILE_NAME));
        let buffer = Buffer::from_content(Content::with_text(TEST_CONTENT));

        save_file_disk(file_id, buffer, PathBuf::from(TEST_PROJECT_ROOT)).await
    }

    fn clear_cache() -> Result<(), std::io::Error> {
        if let Some(env_path) = retrieve_env_path() {
            let cache = env_path.join("recent.cache");
            if cache.exists() {
                fs::remove_file(&cache)?;
            }
        }
        Ok(())
    }

    #[test]
    fn test_get_relative_path() {
        assert_eq!(
            get_relative_path(
                &fake_user!("project/"),
                &fake_user!("project/sub_dir/file.typ")
            ),
            Some(PathBuf::from("/sub_dir/file.typ"))
        );
        assert_eq!(
            get_relative_path(&fake_user!("project/"), &fake_user!("project/file.typ")),
            Some(PathBuf::from("/file.typ"))
        );
        assert_eq!(
            get_relative_path(&fake_user!("project/"), &fake_user!("project/")),
            Some(PathBuf::from("/"))
        );
        assert_eq!(
            get_relative_path(
                &fake_user!("project/"),
                &fake_user!("other_project/file.typ")
            ),
            None
        );
    }

    #[tokio::test]
    async fn test_cache_project() {
        let project_a = ProjectCache::new(fake_user!("project_a/"), None);
        let project_b = ProjectCache::new(
            fake_user!("project_b/"),
            Some(fake_user!("project_b/main.typ")),
        );
        let project_c = ProjectCache::new(fake_user!("project_c/"), None);
        let project_d = ProjectCache::new(
            fake_user!("project_d/"),
            Some(fake_user!("project_d/hello.typ")),
        );
        let project_e = ProjectCache::new(fake_user!("project_e/"), None);
        let project_f = ProjectCache::new(fake_user!("project_f/"), None);

        if clear_cache().is_ok() {
            //cache 2 projects, the latest is 'project_a'
            cache_project(project_a.clone()).await;
            cache_project(project_b.clone()).await;
            assert!(check_cached_projects(0, &project_b));
            assert!(check_cached_projects(1, &project_a));
            //cache 2 more projects, the latest is 'project_d'
            cache_project(project_c.clone()).await;
            cache_project(project_d.clone()).await;
            assert!(check_cached_projects(0, &project_d));
            assert!(check_cached_projects(1, &project_c));
            //re-cache 'project_a'
            cache_project(project_a.clone()).await;
            assert!(check_cached_projects(0, &project_a));
            assert_eq!(get_recent_paths().len(), 4); //a, d, c, b
                                                     //cache 1 more project, the latest is 'project_e'
            cache_project(project_e.clone()).await;
            assert!(check_cached_projects(0, &project_e));
            //cache 1 more project, the latest is 'project_c'
            //but we reach the maximum number of recent projects to cache
            cache_project(project_f.clone()).await;
            assert!(check_cached_projects(0, &project_f));
            assert!(check_cached_projects(1, &project_e));
            assert!(check_cached_projects(2, &project_a));
            assert!(check_cached_projects(3, &project_d));
            assert!(check_cached_projects(4, &project_c));
            assert_eq!(get_recent_paths().len(), MAX_CACHED_PROJECTS);
        }
        assert!(clear_cache().is_ok()); //clear
    }

    #[tokio::test]
    async fn test_save_disk() {
        assert!(create_test_file().await.is_ok());
        assert!(check_file_saved(
            TEST_FILE_NAME,
            String::from(TEST_CONTENT).add("\n")
        ));
    }

    #[tokio::test]
    async fn test_delete_disk() {
        let project_root_path = PathBuf::from(TEST_PROJECT_ROOT);
        let file_path = project_root_path.join(TEST_FILE_NAME);
        //if the file doesn't exist, we create it and then delete it ; if it does, we delete it
        if file_path.try_exists().unwrap_or(false) || create_test_file().await.is_ok() {
            assert!(delete_file_from_disk(
                FileId::new(None, VirtualPath::new(TEST_FILE_NAME)),
                project_root_path
            )
            .is_ok());
        } else {
            panic!("Can't create test file");
        }
        assert!(!file_path.exists());
    }
}
