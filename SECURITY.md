# Security policy

CloudNode holds SSH private keys and passwords and talks to servers it doesn't control. Treat
everything a server sends, every key file and every vault file as potentially malicious.

## Reporting a vulnerability

Please give the maintainers a reasonable chance to investigate and fix a problem before
publishing exploitable details. Contact them through the channels listed on the
[TradeCloud/CloudNode](https://github.com/TradeCloud/CloudNode) repository (GitHub's private
vulnerability reporting when it is enabled), share only a minimal, non-exploitable summary in
public, and ask for a private way to send the rest. Never post a working exploit, a private key,
a password or a vault file in a public issue.

A useful report includes the CloudNode version or commit, the operating system and architecture,
which interface (desktop or console), the affected component, the impact and what an attacker
needs, and exact steps to reproduce. Remove keys, passwords, host names and personal paths from
logs first.

## Scope

- The vault: key derivation, encryption, file handling, anything that could expose or corrupt
  stored keys and passwords.
- SSH: key exchange, host key checking, authentication, channel handling, and anything a server
  can send to crash, hang or exhaust the client.
- Terminal emulation: escape sequences that read or write the user's clipboard or files, open
  URLs, or spoof what the user sees.
- Reading the user's `~/.ssh`: anything that makes CloudNode read private key contents while
  listing keys, follow a path out of the folder unexpectedly, or hang or crash on an odd file.
- Adding a server to `~/.ssh/config`: anything that loses or alters what the file already holds,
  writes outside it, replaces a symbolic link with a file, or lets typed values inject other
  settings.
- Key import and export, and file paths.
- Dependencies, packaging, signing and release integrity.

## Current state

CloudNode is pre-alpha: the vault, SSH sessions and the terminal are not implemented yet
(`ROADMAP.md`). Until they are, CloudNode connects to nothing, and the only file it writes is
`~/.ssh/config`, when the user adds a server with New Server.
