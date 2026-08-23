---
name: user-facing-docs
description: Write README and GitHub Pages copy for operators, not for the design thread. Use when editing README.md, website/, SECURITY.md, or other user-facing documentation.
---
# User-facing documentation

README and the GitHub Pages site (`website/`) are for people who install and
run the app. Issues, PRs, and chat are where design decisions belong.

1. Tell the reader what to run, where it applies, and what they need. Stop
   there. Do not add why an option was rejected, what was discussed, or how
   the files are wired internally.
2. Do not record conversation residue: planned-but-unshipped channels, "this
   is X-only because Y will come later", source-of-truth vs copy, or
   implementation layout, unless the reader must know it to succeed.
3. Keep install and overview copy short. Extra rationale causes documentation
   fatigue; put it in the issue or PR instead.
4. Match the real filename and URL in commands. If a sentence exists only to
   justify a choice made in chat, delete it.
