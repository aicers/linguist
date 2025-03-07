# Linguist - Repository Key Extraction Tool

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
