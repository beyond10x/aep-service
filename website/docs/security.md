---
sidebar_position: 5
title: Security model
---

# Security model

Production SSO and delegated-agent token verification are the principal unfinished security
boundary. The preview verifier accepts one exact token from an environment variable and refuses a
non-loopback bind unless `--allow-insecure-dev-listener` is present. That flag prints a warning and
does not make the verifier production-safe.

Already enforced boundaries include server-owned request identity/time, separate authority and
executor, realm/workspace authorization before semantic dispatch, no public PostgreSQL/provider API,
immutable definition digests, bounded listener resources, and transactional command publication.

Report suspected vulnerabilities through GitHub private vulnerability reporting. Never include
credentials, company data, customer data, or production configuration in a public issue.
