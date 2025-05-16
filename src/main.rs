// SPDX-FileCopyrightText: The ansible-wrapper Authors
// SPDX-License-Identifier: 0BSD

use crate::ansible::galaxy::{parse_installed_collections, parse_installed_roles};
use crate::ansible::model::{GalaxyRequirement, GalaxyRequirementsFile};
use crate::python::model::PyProjectFile;
use fs::File;
use semver::{Version, VersionReq};
use std::collections::BTreeMap;
use std::env::ArgsOs;
use std::ffi::OsString;
use std::fs;
use std::iter::Skip;
use std::path::PathBuf;
use std::process::Command;
use anyhow::Context;
use which::which;

mod ansible;
mod python;

fn main() -> anyhow::Result<()> {
    which("uv").context("[ERROR] You must have 'uv' installed on your system")?;
    which("uvx").context("[ERROR] You must have 'uvx' installed on your system")?;

    let (ansible_command, ansible_arguments) = determine_ansible_command_and_arguments();

    let use_ansible_from_pyproject = ansible_version_is_managed();

    if ansible_command_uses_galaxy_dependencies(&ansible_command) {
        if let Some(requirements_file) = lookup_galaxy_requirements_file() {
            if let Some(galaxy_requirements) = parse_galaxy_requirements(&requirements_file) {
                let mut run_ansible_galaxy_install = false;
                if !galaxy_requirements.collections.is_empty() {
                    let installed_galaxy_collections = parse_installed_collections()?;
                    run_ansible_galaxy_install |= requires_ansible_galaxy_install(
                        &installed_galaxy_collections,
                        &galaxy_requirements.collections,
                    );
                }
                if !run_ansible_galaxy_install && !galaxy_requirements.roles.is_empty() {
                    let installed_galaxy_roles = parse_installed_roles()?;
                    run_ansible_galaxy_install |= requires_ansible_galaxy_install(
                        &installed_galaxy_roles,
                        &galaxy_requirements.roles,
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
                            .status()?
                    } else {
                        Command::new("uvx")
                            .arg("--from")
                            .arg(ansible_core_package(use_ansible_from_pyproject))
                            .arg("ansible-galaxy")
                            .arg("install")
                            .arg("-r")
                            .arg(&requirements_file)
                            .status()?
                    };
                    let exist_code = status.code().unwrap_or(0);
                    assert!(exist_code == 0, "ansible-galaxy was not successful");
                }
            }
        }
    }

    if use_ansible_from_pyproject {
        Command::new("uv")
            .arg("run")
            .arg("--")
            .arg(ansible_command)
            .args(ansible_arguments)
            .status()?;
    } else {
        Command::new("uvx")
            .arg("--from")
            .arg(ansible_core_package(use_ansible_from_pyproject))
            .arg(ansible_command)
            .args(ansible_arguments)
            .status()?;
    }
    
    Ok(())
}

fn ansible_core_package(use_ansible_from_pyproject: bool) -> String {
    Some(use_ansible_from_pyproject)
        .filter(|&managed| !managed)
        .and_then(|_| std::env::var("ANSIBLE_WRAPPER_ANSIBLE_VERSION").ok())
        .map_or_else(|| String::from("ansible-core"), |version| format!("ansible-core=={version}"))
}

fn determine_ansible_command_and_arguments() -> (OsString, Skip<ArgsOs>) {
    let (command, argument_index) = std::env::args_os()
        .next()
        .filter(|command| !PathBuf::from(command).ends_with("ansible-wrapper"))
        .map(|command| (command, 1))
        .or_else(|| {
            std::env::args_os()
                .next()
                .filter(|command| PathBuf::from(command).ends_with("ansible-wrapper"))
                .and_then(|_| {
                    std::env::args_os()
                        .nth(1)
                        .filter(|subcommand| {
                            subcommand == "config"
                                || subcommand == "console"
                                || subcommand == "doc"
                                || subcommand == "galaxy"
                                || subcommand == "inventory"
                                || subcommand == "playbook"
                                || subcommand == "pull"
                                || subcommand == "vault"
                        })
                        .map(|subcommand| {
                            let mut ansible_command = OsString::from("ansible-");
                            ansible_command.push(subcommand);
                            ansible_command
                        })
                        .map(|ansible_command| (ansible_command, 2))
                })
        })
        .unwrap_or_else(|| (OsString::from("ansible"), 1));

    (command, std::env::args_os().skip(argument_index))
}

fn ansible_command_uses_galaxy_dependencies(ansible_command: &OsString) -> bool {
    !std::env::args_os().any(|arg| arg == "--help" || arg == "-h")
        && !std::env::args_os().any(|arg| arg == "--version")
        && (ansible_command == "ansible-playbook"
            || ansible_command == "ansible-console"
            || ansible_command == "ansible-pull")
}

fn ansible_version_is_managed() -> bool {
    fs::read_to_string("pyproject.toml")
        .ok()
        .and_then(|file| toml::from_str::<PyProjectFile>(&file).ok())
        .is_some_and(|pyproject: PyProjectFile| {
            pyproject.project.dependencies.iter().any(|dependency| {
                dependency.starts_with("ansible") || dependency.starts_with("ansible-core")
            })
        })
}

fn lookup_galaxy_requirements_file() -> Option<OsString> {
    std::env::var_os("ANSIBLE_WRAPPER_ANSIBLE_GALAXY_REQUIREMENTS_FILE")
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from("requirements.yml")).filter(|path| path.exists()))
        .or_else(|| Some(PathBuf::from("requirements.yaml")).filter(|path| path.exists()))
        .map(PathBuf::into_os_string)
}

fn parse_galaxy_requirements(requirements: &OsString) -> Option<GalaxyRequirementsFile> {
    File::open(requirements)
        .ok()
        .and_then(|file| serde_yaml_ng::from_reader(file).ok())
}

fn requires_ansible_galaxy_install(
    installed_ansible_collections: &BTreeMap<String, BTreeMap<String, Vec<String>>>,
    ansible_requirements: &Vec<GalaxyRequirement>,
) -> bool {
    for requirement in ansible_requirements {
        let mut found_installed_version = false;
        for installed_collections in installed_ansible_collections.values() {
            found_installed_version |= installed_collections
                .get(&requirement.name)
                .is_some_and(|installed_collections| {
                    installed_collections.iter().any(|installed_version| {
                        installed_version_fulfills_requirement(
                            installed_version,
                            &requirement.version,
                        )
                    })
                });
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
