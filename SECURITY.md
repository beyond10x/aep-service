# Security policy

## Supported status

`aep-service` is a developer preview. Security fixes target the latest commit and latest tagged
preview; no long-term support branch exists yet. The development bearer verifier is not a
production identity implementation.

## Reporting a vulnerability

Please use GitHub's private vulnerability reporting for this repository. Do not open a public issue
for a suspected vulnerability and do not include credentials, tokens, customer data, or production
configuration in any report. We will acknowledge a report as soon as practical, coordinate a fix
privately, and publish remediation information once exposure is contained.

## Preview deployment warning

Keep the service on loopback by default. `--allow-insecure-dev-listener` deliberately makes the
fixed development bearer reachable beyond loopback and is appropriate only behind an isolated,
trusted preview boundary. Rotate the token after any suspected disclosure.

## Build-time dependency advisories

The documentation build currently inherits the published `image-size` denial-of-service advisory
through Docusaurus's MDX loader. It has no upstream-fixed release as of this preview. The affected
parser runs only while building this repository's trusted documentation and is not present in the
service binary or static site. Dependabot tracks the dependency; do not build untrusted image files
with this documentation toolchain.
