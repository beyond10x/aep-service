# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

- Added a framework-neutral HTTP adapter for every AEP service wire v1 command and query route,
  including explicit version negotiation and typed problem responses.
- Added verified human/delegated-agent principals, server-owned command attribution and an
  authorized realm/workspace service-binding seam before semantic dispatch or materialization.
- Pinned EP 0.35.0 and verified the service byte-for-byte against its constructed wire corpus.
- Established the repository, architectural boundaries and governed delivery plan.
