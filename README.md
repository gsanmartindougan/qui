# Qui

Qui is a small Linux desktop app for switching the GitHub SSH account used by Git and GitHub CLI (`gh`). It detects the existing Work and Personal profiles and lets you add more accounts.

## Features

- Switch the GitHub SSH key used by `git@github.com` without `-work` or `-personal` URL suffixes.
- Add profiles with an existing SSH key or create an Ed25519 key from the app.
- Copy the public key so you can add it to GitHub.
- Check the active `gh` login.
- Use the interface in Spanish or English, with the system language detected automatically and a saved in-app switch.
- Keep Git commit author identity controlled by your existing directory-specific Git configuration.

## Requirements

- Linux x86_64.
- `git`, GitHub CLI (`gh`), OpenSSH (`ssh` and `ssh-keygen`).
- Kitty to open interactive `gh` sign-in and key creation; `x-terminal-emulator` is used when available as a fallback.
- An XDG file picker portal (GTK, GNOME, or KDE) for selecting an existing key or destination.

## Run

Download `Qui-x86_64.AppImage` from the [latest release](https://github.com/gsanmartindougan/qui/releases/latest), then:

```sh
chmod +x Qui-x86_64.AppImage
./Qui-x86_64.AppImage
```

For development, install Rust and run:

```sh
cargo run
```

## Add an account

Choose **Add account**, enter a display name and the account's GitHub username, then select an existing private SSH key or create an Ed25519 key. Qui keeps the private key local and displays the `.pub` key so you can copy it to GitHub. Profiles are stored in `$XDG_CONFIG_HOME/qui/profiles.json` (usually `~/.config/qui/profiles.json`).

When you activate a profile, Qui configures SSH for `github.com` and Git to normalize GitHub remotes to `git@github.com:`. It adds an `Include` to `~/.ssh/config` and a global Git include for managed configuration files under `$XDG_CONFIG_HOME/qui/` (usually `~/.config/qui/`). It does not change `user.name`, `user.email`, saved remote URLs, or `gh` credentials. GitHub CLI authentication remains managed by `gh`.

## Check the active accounts

Run these commands inside a repository:

```sh
git remote get-url origin
ssh -T git@github.com
gh auth status --hostname github.com
gh api user --jq .login
git var GIT_AUTHOR_IDENT
```

The SSH greeting identifies the GitHub account selected by the active key. `gh api` shows the account currently authenticated in `gh`. `git var` shows the commit author identity for the current directory; it is separate from the SSH and `gh` accounts.

## Undo Qui's Git and SSH configuration

Remove Qui's `Include` line from `~/.ssh/config` and the global Git `include.path` pointing to `ssh-profile.gitconfig`. You can then delete the managed files in `$XDG_CONFIG_HOME/qui/` (usually `~/.config/qui/`). Your keys and GitHub CLI credentials are not deleted by this cleanup.
