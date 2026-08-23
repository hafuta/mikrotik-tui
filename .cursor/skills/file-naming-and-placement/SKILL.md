---
name: file-naming-and-placement
description: Name and place new files so the path states audience, platform, and role. Use when adding scripts, installers, docs, public URLs, CI helpers, or any file whose name could be read as applying more broadly than it does.
---
# File naming and placement

The path is the contract. People and agents infer who a file is for from its
name and directory. A generic name implies a generic audience.

1. Look at neighboring files and the directory's existing job before choosing a
   name. Do not reuse a folder for a second product (site assets vs app
   installers vs crate sources vs CI).
2. Encode the constraint in the filename when the file is not universal. If
   only Linux users should run it, `scripts/install.sh` is a bad name;
   `scripts/install-linux.sh` is not. Same for OS-specific docs, CI, or
   packaging (`*-macos*`, `*-windows*`) instead of `install`, `setup`, or
   `build` with no qualifier.
3. One documented home. Do not add copies, generate-into hooks, or gitignore
   rules for a parallel path unless that path is a real URL or artifact you
   will show to users.
4. README, site, and CI must use the real filename. If the one-liner looks
   wrong, rename the file; do not alias it silently.

If a future reader on another OS could reasonably think the file is for them,
the name is too broad.
