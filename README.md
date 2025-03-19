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

## Usage

### Cloning Repositories with SSH Authentication

To run `linguist`, you must specify the **path to your SSH private key** as an argument.

```sh
cargo run ~/.ssh/my_custom_rsa_key
```

### Prerequisites

Before running `linguist`, make sure:

* You have GitHub SSH authentication set up.
* Your SSH key is added to GitHub.

  Run the following command to test SSH authentication

  ```sh
  ssh -T git@github.com
  ```

  If you see:
  _"Hi `<your-username>`! You've successfully authenticated.."_

  **Then SSH is working correctly.**

* Your SSH key is added to `ssh-agent`:

  ```sh
  eval "$(ssh-agent -s)"
  ssh-add ~/.ssh/my_custom_rsa_key
  ```

  If authentication issues persist, rerun this command again.

## License

Copyright 2022-2025 ClumL Inc.

Licensed under [Apache License, Version
2.0](https://www.apache.org/licenses/LICENSE-2.0)
(the "License"); you may not use this crate except in compliance with the License.

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See [LICENSE](LICENSE) for
the specific language governing permissions and limitations under the License.
