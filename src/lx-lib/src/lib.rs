pub mod utils;

pub use crate::utils::{is_file_hidden, mode_to_rwx, system_time_to_local_date};
use chrono::NaiveDate;
use derive_builder::Builder;
use std::os::unix::fs::PermissionsExt;
use std::{fs, path};

#[derive(Builder, Debug, PartialEq)]
pub struct DirectoryItem {
    /// Name of the item within the directory
    pub name: String,

    /// Type of the item within the directory
    pub is_dir: bool,

    /// Is item hidden within the directory
    pub is_hidden: bool,

    /// File permissions represented as a num
    pub file_permissions: String,

    /// Time when file was created
    pub created_at: NaiveDate,

    /// Total file size
    pub size: u64,
}

/// Find items within a directory and return back a vector with the directory items.
pub fn find_directory_items(directory: &String) -> Vec<DirectoryItem> {
    let directory = path::Path::new(directory);
    if let Ok(entries) = fs::read_dir(directory) {
        entries
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name().to_str()?.to_owned();
                let is_dir = entry.file_type().ok()?.is_dir();
                let is_hidden = is_file_hidden(entry.file_name());
                let file_permissions = entry.metadata().ok()?.permissions().mode();
                let size = entry.metadata().ok()?.len();
                let created_at = system_time_to_local_date(entry.metadata().ok()?.created().ok()?);

                DirectoryItemBuilder::default()
                    .name(name)
                    .is_dir(is_dir)
                    .is_hidden(is_hidden)
                    .file_permissions(mode_to_rwx(file_permissions))
                    .created_at(created_at)
                    .size(size)
                    .build()
                    .ok()
            })
            .collect()
    } else {
        Vec::new()
    }
}

/// Find items within a directory and subdirectories recursively
pub fn find_directory_items_recursive(directory: &String) -> Vec<DirectoryItem> {
    let mut all_items = Vec::new();
    find_items_recursive(path::Path::new(directory), &mut all_items, "");
    all_items
}

/// Helper function for recursive directory traversal
fn find_items_recursive(dir_path: &path::Path, items: &mut Vec<DirectoryItem>, prefix: &str) {
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name();

            let name_with_prefix = if prefix.is_empty() {
                file_name.to_string_lossy().to_string()
            } else {
                format!("{}/{}", prefix, file_name.to_string_lossy())
            };

            if let Ok(file_type) = entry.file_type() {
                let is_dir = file_type.is_dir();
                let is_hidden = is_file_hidden(file_name);

                if let Ok(metadata) = entry.metadata() {
                    let file_permissions = metadata.permissions().mode();
                    let created_at = match metadata.created() {
                        Ok(time) => system_time_to_local_date(time),
                        Err(_) => chrono::Local::now().date_naive(),
                    };

                    if let Ok(item) = DirectoryItemBuilder::default()
                        .name(name_with_prefix.clone())
                        .is_dir(is_dir)
                        .is_hidden(is_hidden)
                        .file_permissions(mode_to_rwx(file_permissions))
                        .created_at(created_at)
                        .build()
                    {
                        items.push(item);
                    }

                    if is_dir {
                        find_items_recursive(&path, items, &name_with_prefix);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{mode_to_rwx, system_time_to_local_date};
    use crate::{DirectoryItem, find_directory_items};
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::time::SystemTime;

    #[test]
    fn test_find_directory_items_with_static_dir() {
        let temp = TempDir::new().unwrap();

        temp.child("file.txt").touch().unwrap();
        temp.child("subdir").create_dir_all().unwrap();

        let file_metadata = fs::metadata(temp.child("file.txt").path()).unwrap();
        let dir_metadata = fs::metadata(temp.child("subdir").path()).unwrap();

        let file_permissions = file_metadata.permissions();
        let dir_permissions = dir_metadata.permissions();

        let file_mode = file_permissions.mode();
        let dir_mode = dir_permissions.mode();

        let path_as_str = &temp.path().to_str().unwrap().to_owned();
        let mut items = find_directory_items(path_as_str);

        items.sort_by(|a, b| a.name.cmp(&b.name));

        assert_eq!(items.len(), 2);

        let expected: Vec<DirectoryItem> = vec![
            DirectoryItem {
                name: "file.txt".to_string(),
                is_dir: false,
                is_hidden: false,
                file_permissions: mode_to_rwx(file_mode),
                created_at: system_time_to_local_date(SystemTime::now()),
                size: 0,
            },
            DirectoryItem {
                name: "subdir".to_string(),
                is_dir: true,
                is_hidden: false,
                file_permissions: mode_to_rwx(dir_mode),
                created_at: system_time_to_local_date(SystemTime::now()),
                size: 64,
            },
        ];

        assert_eq!(items[0].name, expected[0].name);
        assert_eq!(items[0].is_dir, expected[0].is_dir);
        assert_eq!(items[0].is_hidden, expected[0].is_hidden);
        assert_eq!(items[0].file_permissions, expected[0].file_permissions);
        assert_eq!(items[0].created_at, expected[0].created_at);

        assert_eq!(items[1].name, expected[1].name);
        assert_eq!(items[1].is_dir, expected[1].is_dir);
        assert_eq!(items[1].is_hidden, expected[1].is_hidden);
        assert_eq!(items[1].file_permissions, expected[1].file_permissions);
        assert_eq!(items[1].created_at, expected[1].created_at);
    }

    #[test]
    fn test_find_directory_items_with_hidden_file() {
        let temp = TempDir::new().unwrap();
        temp.child(".file.txt").touch().unwrap();

        let path_as_str = &temp.path().to_str().unwrap().to_owned();
        let mut items = find_directory_items(path_as_str);

        let file_metadata = fs::metadata(temp.child(".file.txt").path()).unwrap();

        let file_permissions = file_metadata.permissions();

        let file_mode = file_permissions.mode();

        items.sort_by(|a, b| a.name.cmp(&b.name));

        assert_eq!(items.len(), 1);

        let expected: Vec<DirectoryItem> = vec![DirectoryItem {
            name: ".file.txt".to_string(),
            is_dir: false,
            is_hidden: true,
            file_permissions: mode_to_rwx(file_mode),
            created_at: system_time_to_local_date(SystemTime::now()),
            size: 0,
        }];

        assert_eq!(items, expected);
    }
}
