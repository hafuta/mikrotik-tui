# Security

MikroTik TUI mutates RouterOS through confirmed actions and property sheets.
It still handles credentials that can be valuable on a local network.

- Use a dedicated RouterOS user with only the permissions required for the
  menus you inspect or edit, plus `api-ssl` access.
- Keep TLS verification enabled. Approve a self-signed fingerprint only
  after comparing it through a trusted channel.
- Treat remembered passwords as sensitive. They are stored in the OS keychain
  when it is available; the file fallback is permission-restricted, not
  encrypted. Prefer a mounted secret file on shared or container hosts.
  Do not enable **Remember password** on a kiosk. TOTP codes are never saved.
- RouterOS entity secrets (PPP passwords, VPN pre-shared keys, RADIUS secrets,
  SNMP communities, user passwords) are masked in tables, inspectors, and
  logs. Saving a properties sheet does not send a still-masked value back.
- The Linux install script downloads GitHub Release archives over HTTPS and
  checks `checksums.txt` before replacing a binary. Review
  `scripts/install-linux.sh` before piping it to a shell.
- Do not attach application logs, profiles, or router exports to public issues
  without reviewing them for network details.

Report vulnerabilities privately to the repository owner. Include the affected
version, reproduction steps, and impact; do not include real credentials.
