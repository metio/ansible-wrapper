// SPDX-FileCopyrightText: The ansible-wrapper Authors
// SPDX-License-Identifier: 0BSD

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(crate) struct AnsibleRequirementsFile {
    pub(crate) collections: Vec<AnsibleRequirement>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct AnsibleRequirement {
    pub(crate) name: String,
    pub(crate) version: String,
}
