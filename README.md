Enpass Rust
===
Command-line prototype to list Enpass database contents

Based on https://github.com/hazcod/enpass-cli

```
Usage: enpass-rust --vault <VAULT> [COMMAND]

Commands:
  list      List the items in the vault
  password
  dump
  help      Print this message or the help of the given subcommand(s)

Options:
  -v, --vault <VAULT>
  -h, --help           Print help
```

Examples
---
`enpass-rust -v $VAULT list`
asks for password and lists index and names of items.

`enpass-rust -v $VAULT password $INDEX`
asks for password and shows user / password for item at $INDEX

`enpass-rust -v $VAULT dump $INDEX`
asks for password and dumps all the not-empty fields for item at $INDEX.
As in the `password` command case, passwords are also processed (decrypted) but also TOTP fields show their current value.
