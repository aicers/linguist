# Linguist - Repository Key Extraction Tool

Linguist checks language files to verify consistency and integrity.

It helps maintain consistency by detecting missing, unused, or mismatched keys
across different language versions. This tool extracts key-value pairs from the
[`frontary`](https://github.com/frontary) repository, which contains predefined
localization texts. Additionally, it processes another internal repository to
compare and validate key usage across multiple language versions.

`linguist` currently supports **two languages**: **English**, and **Korean**

By identifying inconsistencies between these translations, `linguist` ensures
that they remain accurate and complete.

## Prerequisites

Before running `linguist`, make sure:

- You have GitHub SSH authentication set up.
- Your SSH key is added to GitHub.

  Run the following command to test SSH authentication

  ```sh
  ssh -T git@github.com
  ```

  If you see: _"Hi `<your-username>`! You've successfully authenticated.."_

  **Then SSH is working correctly.**

- Your SSH key is added to `ssh-agent`:

  ```sh
  eval "$(ssh-agent -s)"
  ssh-add ~/.ssh/my_custom_rsa_key
  ```

  If authentication issues persist, rerun this command again.

## Usage

```sh
linguist --ssh-key <SSH_KEY_PATH> [--ui-path <UI_PATH>] [--frontary-path \ <FRONTARY_PATH>]
```

### Arguments

<!-- markdownlint-disable -->
| Argument                          | Description                                                   | Required |
|-----------------------------------|---------------------------------------------------------------|----------|
| `--ssh-key <SSH_KEY_PATH>`        | Path to your SSH private key file used for GitHub operations  | Yes      |
| `--ui-path <UI_PATH>`             | Local path of the `aice-web` repo instead of cloning remotely | No       |
| `--frontary-path <FRONTARY_PATH>` | Local path of the `frontary` repo instead of cloning remotely | No       |
<!-- markdownlint-enable -->

#### Notes on Arguments

- The `--ssh-key <SSH_KEY_PATH>` argument:
  - Required for GitHub authentication.
  - Must point to your SSH private key (e.g., `~/.ssh/id_rsa`).
  - Ensure the key is loaded into your SSH agent before running.

- The `--ui-path <UI_PATH>` argument:
  - Optional; if provided, uses this local directory as the aice-web repository.
  - If omitted, linguist will clone aice-web from the default remote URL.

- The `--frontary-path <FRONTARY_PATH>` argument:
  - Optional; if provided, uses this local directory as the frontary repository.
  - If omitted, linguist will clone frontary from the default remote URL.

## License

Copyright 2025 ClumL Inc.

Licensed under [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
(the "License"); you may not use this crate except in compliance with the License.

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See [LICENSE](LICENSE) for
the specific language governing permissions and limitations under the License.
