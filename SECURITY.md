# Security

MikroTik TUI is read-only in its initial release, but it handles credentials
that can be valuable on a local network.

- Use a dedicated RouterOS user with only the permissions required to read the
  requested resources and access REST.
- Keep HTTPS verification enabled. Approve a self-signed fingerprint only
  after comparing it through a trusted channel.
- Treat the local credential file as sensitive. It is permission-restricted,
  not encrypted. Prefer a mounted secret file in shared/container hosts.
- Do not attach application logs, profiles, or router exports to public issues
  without reviewing them for network details.

Report vulnerabilities privately to the repository owner. Include the affected
version, reproduction steps, and impact; do not include real credentials.
