# Contributing

This repository is in a focused developer preview. Issues are welcome for reproducible defects,
documentation gaps, and concrete use cases. External pull requests are not accepted yet while the
wire, storage, and contribution boundaries settle; please open an issue before investing in a
patch.

For local work, read `AGENTS.md`, use Rust 1.85 or newer, and run:

```console
task check
task site-build
```

Set `ENTITY_POSTGRES_URL` to a disposable PostgreSQL database to run the durable transaction tests.
Never use production data. Generated OpenAPI comes from the AEP-owned route catalog and DTO schemas;
do not hand-edit a generated JSON file or restate the contract in documentation.

By participating, follow the [Contributor Covenant](CODE_OF_CONDUCT.md). All contributions are
licensed under Apache-2.0.
