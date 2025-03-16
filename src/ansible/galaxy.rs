// SPDX-FileCopyrightText: The ansible-wrapper Authors
// SPDX-License-Identifier: 0BSD

use crate::ansible::model::GalaxyInstallInfoFile;
use std::collections::BTreeMap;
use std::fs::File;
use std::path::PathBuf;

pub(crate) fn parse_installed_collections()
-> BTreeMap<String, BTreeMap<String, Vec<String>>> {
    let collections_path = std::env::var_os("ANSIBLE_COLLECTIONS_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            ini::Ini::load_from_file("ansible.cfg")
                .ok()
                .and_then(|config| {
                    config
                        .section(Some("defaults"))
                        .and_then(|section| section.get("collections_path"))
                        .map(PathBuf::from)
                })
        })
        .map(|path| path.join("ansible_collections"));

    let mut paths_to_check: Vec<PathBuf> = vec![];
    if let Some(collections_path_overwrite) = collections_path {
        paths_to_check.push(collections_path_overwrite);
    } else {
        // add defaults paths
        // https://docs.ansible.com/ansible/latest/reference_appendices/config.html#collections-paths
        if let Some(path) = std::env::var_os("ANSIBLE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::home_dir().map(|home_dir| home_dir.join(".ansible")))
            .map(|path| path.join("collections/ansible_collections"))
        {
            paths_to_check.push(path);
        }
        paths_to_check.push(PathBuf::from("/usr/share/ansible/collections/ansible_collections"));
    }

    let mut installed_collections: BTreeMap<String, BTreeMap<String, Vec<String>>> =
        BTreeMap::new();
    for collection_path in paths_to_check {
        if collection_path.exists() {
            let paths = std::fs::read_dir(&collection_path).expect("Failed to read directory");
            for path in paths {
                let entry = path.expect("failed to read directory entry");
                if let Some(extension) = entry.path().extension() {
                    if extension == "info" {
                        let galaxy_info = entry.path().join("GALAXY.yml");
                        if galaxy_info.exists() {
                            let file = File::open(galaxy_info).expect("file should open read only");
                            let info: GalaxyInstallInfoFile = serde_yaml_ng::from_reader(file)
                                .expect("[ERROR] Cannot parse GALAXY.yml file");
                            installed_collections
                                .entry(collection_path.to_string_lossy().into_owned())
                                .or_insert_with(BTreeMap::new)
                                .entry(
                                    entry
                                        .file_name()
                                        .to_string_lossy()
                                        .into_owned()
                                        .replace(&format!("-{}.info", info.version), ""),
                                )
                                .or_insert_with(Vec::new)
                                .push(info.version);
                        }
                    }
                }
            }
        }
    }

    installed_collections
}

pub(crate) fn parse_installed_roles() -> BTreeMap<String, BTreeMap<String, Vec<String>>>
{
    let roles_path = std::env::var_os("ANSIBLE_ROLES_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            ini::Ini::load_from_file("ansible.cfg")
                .ok()
                .and_then(|config| {
                    config
                        .section(Some("defaults"))
                        .and_then(|section| section.get("roles_path"))
                        .map(PathBuf::from)
                })
        });

    let mut paths_to_check: Vec<PathBuf> = vec![];
    if let Some(roles_path_overwrite) = roles_path {
        paths_to_check.push(roles_path_overwrite);
    } else {
        // add defaults paths
        // https://docs.ansible.com/ansible/latest/galaxy/user_guide.html#setting-where-to-install-roles
        if let Some(path) = std::env::var_os("ANSIBLE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::home_dir().map(|home_dir| home_dir.join(".ansible")))
            .map(|path| path.join("roles"))
        {
            paths_to_check.push(path);
        }
        paths_to_check.push(PathBuf::from("/usr/share/ansible/roles"));
        paths_to_check.push(PathBuf::from("/etc/ansible/roles"));
    }

    let mut installed_roles: BTreeMap<String, BTreeMap<String, Vec<String>>> =
        BTreeMap::new();
    for role_path in paths_to_check {
        if role_path.exists() {
            let paths = std::fs::read_dir(&role_path).expect("Failed to read directory");
            for path in paths {
                let entry = path.expect("failed to read directory entry");
                let galaxy_info = entry.path().join("meta/.galaxy_install_info");
                if galaxy_info.exists() {
                    let file = File::open(galaxy_info).expect("file should open read only");
                    let info: GalaxyInstallInfoFile = serde_yaml_ng::from_reader(file)
                        .expect("[ERROR] Cannot parse output of 'ansible-galaxy collection list'");
                    installed_roles
                        .entry(role_path.to_string_lossy().into_owned())
                        .or_insert_with(BTreeMap::new)
                        .entry(entry.file_name().to_string_lossy().into_owned())
                        .or_insert_with(Vec::new)
                        .push(info.version);
                }
            }
        }
    }

    installed_roles
}
