// SPDX-FileCopyrightText: The ansible-wrapper Authors
// SPDX-License-Identifier: 0BSD

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GalaxyRequirementsFile {
    pub(crate) collections: Vec<GalaxyRequirement>,
    pub(crate) roles: Vec<GalaxyRequirement>,
}

#[derive(Deserialize, Debug)]
pub struct GalaxyRequirement {
    pub(crate) name: String,
    pub(crate) version: String,
}

#[derive(Deserialize, Debug)]
pub struct GalaxyInstallInfoFile {
    pub(crate) version: String,
}
