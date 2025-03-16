// SPDX-FileCopyrightText: The ansible-wrapper Authors
// SPDX-License-Identifier: 0BSD

use crate::ansible::galaxy::{parse_installed_collections, parse_installed_roles};
use crate::ansible::model::{GalaxyRequirement, GalaxyRequirementsFile};
use fs::File;
use semver::{Version, VersionReq};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::Command;
use which::which;

mod ansible;

fn main() {
    match std::env::args_os().nth(0) {
        None => {
            panic!("Cannot determine ansible command");
        }
        Some(command) => {
            run_preflight_checks();

            if command == "ansible-playbook" {
                if !user_wants_help() && !user_wants_version() {
                    if let Some(requirements_file) = lookup_requirements_file() {
                        let ansible_requirements = parse_ansible_requirements(&requirements_file);
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
                            let status = Command::new("uv")
                                .arg("run")
                                .arg("--")
                                .arg("ansible-galaxy")
                                .arg("install")
                                .arg("-r")
                                .arg(&requirements_file)
                                .status()
                                .expect("Process to finish with output");
                            let exist_code = status.code().expect("Process to return its exist code");
                            if exist_code != 0 {
                                panic!("ansible-galaxy was not successful")
                            }
                        }
                    }
                }
            }

            Command::new("uv")
                .arg("run")
                .arg("--")
                .arg(command)
                .args(std::env::args_os().skip(1))
                .status()
                .expect("ansible command failed to start");
        }
    }
}

fn user_wants_help() -> bool {
    std::env::args_os().any(|arg| arg == "--help" || arg == "-h")
}

fn user_wants_version() -> bool {
    std::env::args_os().any(|arg| arg == "--version")
}

fn lookup_requirements_file() -> Option<OsString> {
    let requirements_file = std::env::var_os("ANSIBLE_WRAPPER_ANSIBLE_GALAXY_REQUIREMENTS_FILE");

    if let Some(file) = &requirements_file {
        if Path::new(file).exists() {
            return requirements_file;
        }
    } else {
        let default_spelling = OsString::from("requirements.yml");
        if Path::new(&default_spelling).exists() {
            return Some(default_spelling);
        }

        let alternative_spelling = OsString::from("requirements.yaml");
        if Path::new(&alternative_spelling).exists() {
            return Some(alternative_spelling);
        }
    }

    None
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
                .map(|installed_collections| installed_collections.iter()
                    .any(|installed_version| installed_version_fulfills_requirement(installed_version, &requirement.version)))
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
    let requirement =
        VersionReq::parse(&wanted.replace("==", "=")).expect("a valid SemVer version requirement");
    let version = Version::parse(installed).expect("a valid SemVer version");
    requirement.matches(&version)
}

fn parse_ansible_requirements(requirements: &OsString) -> GalaxyRequirementsFile {
    let file = File::open(requirements).expect("file should open read only");
    serde_yaml_ng::from_reader(file)
        .expect("[ERROR] Cannot parse output of 'ansible-galaxy collection list'")
}

fn run_preflight_checks() {
    which("uv").expect("[ERROR] You need to install 'uv' first");
    Path::new("pyproject.toml")
        .try_exists()
        .expect("[ERROR] You need to create a 'pyproject.toml' file first");
}
