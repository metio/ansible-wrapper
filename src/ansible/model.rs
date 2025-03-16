// SPDX-FileCopyrightText: The ansible-wrapper Authors
// SPDX-License-Identifier: 0BSD

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(crate) struct GalaxyRequirementsFile {
    pub(crate) collections: Vec<GalaxyRequirement>,
    pub(crate) roles: Vec<GalaxyRequirement>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct GalaxyRequirement {
    pub(crate) name: String,
    pub(crate) version: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct GalaxyInstallInfoFile {
    pub(crate) version: String,
}
