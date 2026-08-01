# openaleph-tui

This is a text UI for [openaleph](https://www.github.com/openaleph/openaleph). It allows one to monitor the status of an OpenAleph instance in real time. It uses the Aleph API to retrieve this information and shows roughly the same data one would see on OpenAleph's status page.

![openaleph-tui](aleph-tui.png)
![openaleph-tui with profile selection menu open](aleph-tui-profile-selector.png)

## Installation

Check the individual [release pages](https://github.com/stchris/aleph-tui/releases) for instructions.

On mac OS:

```sh
brew install stchris/homebrew-formulae/openaleph-tui
```

## Configuration

`openaleph-tui` assumes the configuration to live in `$HOME/.config/openaleph-tui.toml`. Existing configuration at `$HOME/.config/aleph-tui.toml` is migrated automatically. It allows you to connect to multiple OpenAleph instances through `profiles`. Here is an example file:

```toml
default = "local"

[profiles]
  [profiles.local]
  url = "http://localhost:8080/"
  token = "<YOUR API TOKEN HERE>"

  [profiles.my-aleph]
  url = "https://my.aleph.instance/"
  token = "<YOUR API TOKEN HERE>"
```

Find your personal API token by going to `/settings` on your OpenAleph instance and copying the value from the `API Secret Access Key` field.

`default` defines the profile to be loaded when `openaleph-tui` starts up.
