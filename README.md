# Sticky variables

![Crates.io Version](https://img.shields.io/crates/v/stickyvar)

A little utility to keep environment variables around permanently\* and share them between sessions. It does this by storing them in an sqlite database.

Should support any sufficiently POSIX-y shell, as well as Nushell.

## Installation

### From crates.io

```sh
cargo install stickyvar
```

### Hemorrhaging-edge releases

Go to the latest run of the [build](https://github.com/ysthakur/stickyvar/actions/workflows/build.yml) workflow and download the zip file for your platform:

- `ubuntu-latest` should work for Linux distros
- I'll let you figure out what `windows-latest` and `macos-latest` are for

Inside the zip file should be the executable (yes, that's the only thing inside the zip, blame GitHub for the unnecessary archiving).

### Mildly-prickly-edge releases

Go to the latest release on the [Releases](https://github.com/ysthakur/stickyvar/releases) page and download one of the assets there. I screwed up the 0.1.0 release, though, so there's no assets for that one. Hopefully future releases will.

### Build it yourself

```sh
git clone git@github.com:ysthakur/stickyvar.git
cargo install --path stickyvar
```

## Setup

### POSIX-y shells

If you're using a sufficiently POSIX-y shell, such as Bash (get a better shell) or Zsh, then you can put the following into your `.bashrc` or `.zshrc` or whatever:

```sh
eval "$(stickyvar init sh)"
```

### Nushell

If you're using Nushell, pat yourself on the back for your great taste in shells, then put the following in `env.nu`:

```nu
stickyvar init nu | save --force $"($nu.cache-dir)/stickyvar.nu"
```

It'll create a file `stickyvar.nu` every time you open Nushell. Alternatively, you can also manually run that snippet and generate the setup script just once.

Then, put the following in `config.nu` to actually load the module from the setup script:

```nu
use $"($nu.cache-dir)/stickyvar.nu" sv
```

## Usage

The init script generated earlier should've added an `sv` function/module with the following subcommands:

- `sv set FOO BAR` will set an environment variable `FOO` to `BAR` and add it to the sticky variable database
- `sv load FOO` will grab the value of `FOO` from the sticky variable database, then set the environment variable `FOO` to that in your current shell
- `sv load` (no extra arguments) will load and set all variables from the sticky variable database
- `sv list` will list names and values of sticky variables in the database
  - In Nushell, this actually just opens the database, so it also includes the last modified time of each variable (as seconds since Unix epoch)
- `sv del FOO` will delete `FOO` from the sticky variable database and also unset the environment variable `FOO`

To get help:

- Run `sv` in POSIX-compliant shells
- Run `help sv` in Nushell

### Changing where variables are stored

You can set the environment variable `STICKY_VAR_DB` to control which file you want variables to be stored in. The file doesn't need to exist, but the directory it's in does. You can then check that `stickyvar` is using the correct path using `stickyvar db-path`.

Here's an example:

```sh
> stickyvar db-path
/home/ysthakur/.local/state/stickyvar/sticky-var.db
> $env.STICKY_VAR_DB = "foo.db"
> stickyvar db-path
foo.db
```

### Sharing variables between two sessions

Suppose you have two sessions that you want to share environment variables between, but you don't want any other sessions to get the same variables. You can handle this by having both sessions use the same database file, different from the default one.

If you set the environment variable `STICKY_VAR_DB` to, say, `~/foo.db` in both sessions, they'll both use variables from there, but other sessions will continue using the default database.

[![demo](https://asciinema.org/a/783019.svg)](https://asciinema.org/a/783019)

---

TODOs:

- Pre-prompt hooks for Zsh and Nushell to update variables?
  - Not sure how much time it would take to load all sticky variables every prompt
  - Need to determine which variables to update? Possibly complicated
- Install shell hook to run every time any environment variable is set
  - Record the last time an environment variable was set in a session-specific database
  - Why? `sync` subcommand that reads all sticky vars and only updates env vars that are
    newer than the found sticky vars
  - Not sure how useful this would be
- Set normal shell variables in Nushell rather than just environment variables. Typing `$env.foo` is annoying, `$foo` is easier

<sub>*Nothing in this world is permanent. They're stored in an sqlite database (just a file), and this file could always be deleted or the hard drive could be corrupted by solar radiation or the computer could die from water damage after protecting you from a shark attack.</sub>
