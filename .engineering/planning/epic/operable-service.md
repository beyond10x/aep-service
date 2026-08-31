---
format: aep.planning-md/1
id: epic:operable-service
kind: epic
status: draft
title: An operable service rather than a library demo
summary: Compose, deploy, observe, back up and restore the central service without exposing data or database authority.
relations:
- decomposes: initiative:central-aep-authority
- serves: vision:O1
- serves: vision:O6
revision: 1
---
# Epic: An operable service rather than a library demo

## Outcome

The application runs as one supportable service with explicit configuration, safe database
operation, useful health signals and security/audit observability.

## Scope

Rust server composition, HTTP lifecycle, configuration, migrations, backup/restore, readiness,
metrics, structured logs and operational runbooks.

## Done When

An operator can deploy, upgrade, observe, back up and restore the service without granting clients
database access or learning restricted entity content from telemetry.

