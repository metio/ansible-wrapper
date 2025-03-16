// SPDX-FileCopyrightText: The ansible-wrapper Authors
// SPDX-License-Identifier: 0BSD

use crate::ansible::galaxy::{parse_installed_collections, parse_installed_roles};
use crate::ansible::model::{GalaxyRequirement, GalaxyRequirementsFile};
use crate::python::model::PyProjectFile;
use fs::File;
use semver::{Version, VersionReq};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use which::which;

mod ansible;
mod python;

fn main() {
    match std::env::args_os().nth(0) {
        None => {
            panic!("Cannot determine ansible command");
        }
        Some(command) => {
            run_preflight_checks();

            let use_ansible_from_pyproject = ansible_version_is_managed();
            let ansible_core_dependency: String = Some(use_ansible_from_pyproject)
                .filter(|&managed| !managed)
                .and_then(|_| std::env::var("ANSIBLE_WRAPPER_ANSIBLE_VERSION").ok())
                .map(|version| format!("ansible-core=={}", version))
                .unwrap_or(String::from("ansible-core"));

            if !user_wants_help() && !user_wants_version() {
                if command == "ansible-playbook" {
                    if let Some(requirements_file) = lookup_requirements_file() {
                        if let Some(ansible_requirements) =
                            parse_ansible_requirements(&requirements_file)
                        {
                            let mut run_ansible_galaxy_install = false;
                            if ansible_requirements.collections.len() > 0 {
                                let installed_ansible_collections = parse_installed_collections();
                                run_ansible_galaxy_install |= requires_ansible_galaxy_install(
                                    installed_ansible_collections,
                                    &ansible_requirements.collections,
                                );
                            }
                            if ansible_requirements.roles.len() > 0 {
                                let installed_ansible_roles = parse_installed_roles();
                                run_ansible_galaxy_install |= requires_ansible_galaxy_install(
                                    installed_ansible_roles,
                                    &ansible_requirements.roles,
                                );
                            }
                            if run_ansible_galaxy_install {
                                let status = if use_ansible_from_pyproject {
                                    Command::new("uv")
                                        .arg("run")
                                        .arg("--")
                                        .arg("ansible-galaxy")
                                        .arg("install")
                                        .arg("-r")
                                        .arg(&requirements_file)
                                        .status()
                                        .expect("Process to finish with output")
                                } else {
                                    Command::new("uvx")
                                        .arg("--from")
                                        .arg(&ansible_core_dependency)
                                        .arg("ansible-galaxy")
                                        .arg("install")
                                        .arg("-r")
                                        .arg(&requirements_file)
                                        .status()
                                        .expect("Process to finish with output")
                                };
                                let exist_code =
                                    status.code().expect("Process to return its exist code");
                                if exist_code != 0 {
                                    panic!("ansible-galaxy was not successful")
                                }
                            }
                        }
                    }
                }
            }

            if use_ansible_from_pyproject {
                Command::new("uv")
                    .arg("run")
                    .arg("--")
                    .arg(command)
                    .args(std::env::args_os().skip(1))
                    .status()
                    .expect("ansible command failed to start");
            } else {
                Command::new("uvx")
                    .arg("--from")
                    .arg(&ansible_core_dependency)
                    .arg(command)
                    .args(std::env::args_os().skip(1))
                    .status()
                    .expect("ansible command failed to start");
            }
        }
    }
}

fn ansible_version_is_managed() -> bool {
    fs::read_to_string("pyproject.toml")
        .ok()
        .and_then(|file| toml::from_str::<PyProjectFile>(&file).ok())
        .map(|pyproject: PyProjectFile| {
            pyproject.project.dependencies.iter().any(|dependency| {
                dependency.starts_with("ansible") || dependency.starts_with("ansible-core")
            })
        })
        .unwrap_or(false)
}

fn user_wants_help() -> bool {
    std::env::args_os().any(|arg| arg == "--help" || arg == "-h")
}

fn user_wants_version() -> bool {
    std::env::args_os().any(|arg| arg == "--version")
}

fn lookup_requirements_file() -> Option<OsString> {
    std::env::var_os("ANSIBLE_WRAPPER_ANSIBLE_GALAXY_REQUIREMENTS_FILE")
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from("requirements.yml")).filter(|path| path.exists()))
        .or_else(|| Some(PathBuf::from("requirements.yaml")).filter(|path| path.exists()))
        .map(|path| path.into_os_string())
}

fn parse_ansible_requirements(requirements: &OsString) -> Option<GalaxyRequirementsFile> {
    File::open(requirements)
        .ok()
        .and_then(|file| serde_yaml_ng::from_reader(file).ok())
}

fn requires_ansible_galaxy_install(
    installed_ansible_collections: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    ansible_requirements: &Vec<GalaxyRequirement>,
) -> bool {
    for requirement in ansible_requirements {
        let mut found_installed_version = false;
        for (_, installed_collections) in &installed_ansible_collections {
            found_installed_version |= installed_collections
                .get(&requirement.name)
                .map(|installed_collections| {
                    installed_collections.iter().any(|installed_version| {
                        installed_version_fulfills_requirement(
                            installed_version,
                            &requirement.version,
                        )
                    })
                })
                .unwrap_or(false);
        }
        if !found_installed_version {
            return true;
        }
    }
    false
}

fn installed_version_fulfills_requirement(installed: &str, wanted: &str) -> bool {
    if wanted == "*" {
        return true;
    }
    VersionReq::parse(&wanted.replace("==", "="))
        .ok()
        .and_then(|requirement| {
            Version::parse(installed)
                .ok()
                .map(|version| requirement.matches(&version))
        })
        .unwrap_or(false)
}

fn run_preflight_checks() {
    which("uv").expect("[ERROR] You need to install 'uv' first");
}
