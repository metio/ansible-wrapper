// SPDX-FileCopyrightText: The ansible-wrapper Authors
// SPDX-License-Identifier: 0BSD

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PyProjectFile {
    pub(crate) project: Project,
}

#[derive(Deserialize, Debug)]
pub struct Project {
    pub(crate) dependencies: Vec<String>,
}
