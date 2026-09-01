# Vision — one governed authority for engineering decisions

Engineering work is currently easy to inspect inside one repository and difficult to understand
across repositories. The durable record is fragmented among Markdown planning trees, Jira intake,
database backends and generated views. Stable identities, dependencies, approvals and refusal
history stop at repository boundaries.

`aep-service` makes the complete AEP record centrally authoritative without making a generic issue
tracker. Humans, agents and integrations submit semantic commands through the same authenticated
boundary. The service derives who authorised an action and what executed it, checks access and
protocol rules independently, and records the resulting decision transactionally.

Repositories retain their useful local surface. Markdown is a deterministic projection that can be
committed for review or generated on demand; it is never a second write path. Jira remains an intake
and external-coordination system whose reports relate to internal engineering entities rather than
owning their lifecycle.

The service succeeds when:

- `protocol` can operate from any adopting repository against one authenticated authority;
- an agent can never exercise more authority than its owner delegated;
- every accepted or refused authenticated command is attributable and replayable against its
  governing AEP definition bundle;
- cross-repository dependencies and objective chains are queryable without parsing Git trees;
- authorized snapshots reproduce repository Markdown byte-for-byte; and
- no client needs PostgreSQL credentials or a raw Entity Runtime store interface.

The service does not grow Jira's product surface: no sprint UI, comments, notifications, marketplace,
customer-service queues or arbitrary workflow editor. Presentation remains a projection or an
integration unless it is necessary to preserve the governed command/query contract.

