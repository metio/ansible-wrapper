<!--
SPDX-FileCopyrightText: The ansible-wrapper Authors
SPDX-License-Identifier: 0BSD
 -->

# ansible-wrapper

`ansible-wrapper` is a wrapper various Ansible commands using `uv` to automatically download both Python dependencies and required Ansible collections.

## Features

- Automatically downloads an appropriate Python version specified in `.python-version`. If no such file exists, no Python version will be installed and whatever is currently installed on your system is being used. See https://docs.astral.sh/uv/concepts/python-versions/#python-version-files for more details.
- Automatically downloads an appropriate Ansible version specified in `pyproject.toml`. If no such file exists or no dependency to either `ansible` or `ansible-core` is declared, we are going to use the latest available Ansible version or the version set in `ANSIBLE_WRAPPER_ANSIBLE_VERSION`. See https://packaging.python.org/en/latest/guides/writing-pyproject-toml/ for more details.
- Automatically downloads all required Ansible collections and roles specified in `requirements.yml`, `requirements.yaml`, or what `ANSIBLE_WRAPPER_ANSIBLE_GALAXY_REQUIREMENTS_FILE` points to. If no such file exists, we are not installing/managing Ansible collections and roles. See https://docs.ansible.com/ansible/latest/galaxy/user_guide.html#installing-roles-and-collections-from-the-same-requirements-yml-file for more details.

## Installation

Run `cargo install ansible-wrapper` on your system or download the binary from the releases page and put it somewhere on your system. Once installed, either define symlinks to Ansible CLI commands like this:

```shell
ln -s /path/to/ansible-wrapper ~/.local/bin/ansible
ln -s /path/to/ansible-wrapper ~/.local/bin/ansible-config
ln -s /path/to/ansible-wrapper ~/.local/bin/ansible-console
ln -s /path/to/ansible-wrapper ~/.local/bin/ansible-doc
ln -s /path/to/ansible-wrapper ~/.local/bin/ansible-galaxy
ln -s /path/to/ansible-wrapper ~/.local/bin/ansible-inventory
ln -s /path/to/ansible-wrapper ~/.local/bin/ansible-playbook
ln -s /path/to/ansible-wrapper ~/.local/bin/ansible-pull
ln -s /path/to/ansible-wrapper ~/.local/bin/ansible-vault
```

Or create shell aliases like this:

```shell
alias ansible='/path/to/ansible-wrapper'
alias ansible-config='/path/to/ansible-wrapper config'
alias ansible-console='/path/to/ansible-wrapper console'
alias ansible-doc='/path/to/ansible-wrapper doc'
alias ansible-galaxy='/path/to/ansible-wrapper galaxy'
alias ansible-galaxy='/path/to/ansible-wrapper inventory'
alias ansible-playbook='/path/to/ansible-wrapper playbook'
alias ansible-playbook='/path/to/ansible-wrapper pull'
alias ansible-vault='/path/to/ansible-wrapper vault'
```

Or simply call the wrapper directly like this:

```shell
/path/to/ansible-wrapper ...
/path/to/ansible-wrapper config ...
/path/to/ansible-wrapper console ...
/path/to/ansible-wrapper doc ...
/path/to/ansible-wrapper galaxy ...
/path/to/ansible-wrapper inventory ...
/path/to/ansible-wrapper playbook ...
/path/to/ansible-wrapper pull ...
/path/to/ansible-wrapper vault ...
```
