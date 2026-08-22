# Security

MikroTik TUI mutates RouterOS through confirmed actions and property sheets.
It still handles credentials that can be valuable on a local network.

- Use a dedicated RouterOS user with only the permissions required for the
  menus you inspect or edit, plus REST (`www-ssl`) access.
- Keep HTTPS verification enabled. Approve a self-signed fingerprint only
  after comparing it through a trusted channel.
- Treat the local credential file as sensitive. It is permission-restricted,
  not encrypted. Prefer a mounted secret file in shared/container hosts.
- RouterOS entity secrets (PPP passwords, VPN pre-shared keys, RADIUS secrets,
  SNMP communities, user passwords) are masked in tables, inspectors, and
  logs. Saving a properties sheet does not send a still-masked value back.
- Do not attach application logs, profiles, or router exports to public issues
  without reviewing them for network details.

Report vulnerabilities privately to the repository owner. Include the affected
version, reproduction steps, and impact; do not include real credentials.
