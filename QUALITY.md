# Quality checks

The thin GitHub callers invoke `rptadv-shared-library-workflows`. Pushes run
format, lint, and static analysis; pull requests run the full native Debian 13
amd64/arm64 gate, including packaging and staged installation. The amd64 gate
requires 100% production line and branch coverage. Releases use a merged main
revision whose pull request passed that gate.

Local focused checks use the freshly pulled maintained quality image through
`tools/run-in-quality-container.sh`. `make lint static-analysis`, `make docs`,
and `make test` are the relevant targets. No node deployment is part of this
repository's library validation.
