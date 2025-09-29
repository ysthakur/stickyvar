# Sticky variables

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
