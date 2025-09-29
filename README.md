# Sticky variables

A little utility to keep environment variables around permanently\* and share them between sessions.

Should support any sufficiently POSIX-y shell, as well as Nushell.

## Installation

### Hemorrhaging-edge releases

Go to the latest run of the [build](https://github.com/ysthakur/stickyvar/actions/workflows/build.yml) workflow and download the zip file for your platform:

- `ubuntu-latest` should work for Linux distros
- I'll let you figure out what `windows-latest` and `macos-latest` are for

Inside the zip file should be the executable (yes, that's the only thing inside the zip, blame GitHub for the unnecessary archiving).

## Setup

### POSIX-y shells

If you're using a sufficiently POSIX-y shell, such as Bash (get a better shell) or Zsh, then

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
