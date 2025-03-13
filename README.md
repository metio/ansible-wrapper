<!--
SPDX-FileCopyrightText: The ansible-wrapper Authors
SPDX-License-Identifier: 0BSD
 -->

# ansible-wrapper

`ansible-wrapper` is a wrapper various Ansible commands using `uv` to automatically download both Python dependencies and required Ansible collections.

## Features

- Automatically downloads an appropriate Python version (using `.python-version`)
- Automatically downloads an appropriate Ansible version (using `pyproject.toml`)
- Automatically downloads all required Ansible collections (using `requirements.yml`, `requirements.yaml`, or what `ANSIBLE_WRAPPER_ANSIBLE_GALAXY_REQUIREMENTS_FILE` points to)

## Installation

Download the binary from the releases page, put it somewhere on your system and define symlinks to Ansible CLI commands:

```shell
ln -s /path/to/ansible-wrapper ~/.local/bin/ansible
ln -s /path/to/ansible-wrapper ~/.local/bin/ansible-doc
ln -s /path/to/ansible-wrapper ~/.local/bin/ansible-playbook
ln -s /path/to/ansible-wrapper ~/.local/bin/ansible-vault
```
