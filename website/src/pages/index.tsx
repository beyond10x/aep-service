import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';

import styles from './index.module.css';

const authorityFlow = [
  {number: '01', label: 'INTENT', title: 'Semantic command', detail: 'Create, move, approve, relate'},
  {number: '02', label: 'TRUST', title: 'Verified context', detail: 'Authority, executor, realm, grants'},
  {number: '03', label: 'DECIDE', title: 'One transaction', detail: 'Rules, revision, state, evidence'},
  {number: '04', label: 'RECORD', title: 'Attributable history', detail: 'Result, events, audit, refusal'},
];

const capabilities = [
  {
    label: 'SEMANTIC API',
    title: 'Submit intent, not storage mutations.',
    text: 'Clients ask to create, approve, relate, or move an engineering entity. SQL, provider batches, and generic status writes never become public API.',
  },
  {
    label: 'TRANSACTIONAL AUTHORITY',
    title: 'A decision lands whole—or not at all.',
    text: 'State, revisions, relations, events, audit, idempotency, and refusals share one PostgreSQL command boundary.',
  },
  {
    label: 'HUMANS + AGENTS',
    title: 'Who authorized work stays distinct from who ran it.',
    text: 'Authority and executor are separate facts. Delegation is designed to narrow an owner’s grants, never expand them.',
  },
  {
    label: 'ONE SOURCE',
    title: 'Every useful view is a projection.',
    text: 'Repository Markdown, dashboards, MCP tools, and oversight views can all project the same central entity and activity record.',
  },
];

const available = [
  'Versioned AEP command and query wire',
  'Fresh PostgreSQL transactions and indexed reads',
  'Immutable definition-bundle pinning',
  'Typed refusals, idempotency, limits, and probes',
  'Derived OpenAPI, binaries, and multiarch OCI image',
];

const next = [
  'Production SSO and delegated-agent token verification',
  'Realm provisioning and bundle activation control plane',
  'Authorized Markdown and dashboard projections',
  'MCP and remote protocol client surfaces',
  'Production observability, backup, and recovery proof',
];

function Status({children}: {children: ReactNode}): ReactNode {
  return <span className={styles.status}><i aria-hidden="true" />{children}</span>;
}

function AuthorityDiagram(): ReactNode {
  return (
    <div className={styles.authorityDiagram} aria-label="A semantic request becomes one attributable transaction">
      <div className={styles.diagramBar}>
        <span>AEP / AUTHORITY TRACE</span>
        <span className={styles.traceId}>0198F03A · ACCEPTED</span>
      </div>
      <div className={styles.actorRow}>
        <div><span>AUTHORITY</span><strong>human:alice</strong></div>
        <div className={styles.delegation}><span>EXECUTOR</span><strong>agent:planner</strong></div>
      </div>
      <div className={styles.commandBlock}>
        <div className={styles.commandHead}>
          <span>COMMAND</span><code>aep.status.move/v1</code>
        </div>
        <div className={styles.commandBody}>
          <p><span>target</span><b>story:api-reference</b></p>
          <p><span>from</span><b>active</b></p>
          <p><span>to</span><strong>implemented</strong></p>
        </div>
      </div>
      <div className={styles.decisionLine} aria-hidden="true"><span /><b>ATOMIC DECISION</b><span /></div>
      <div className={styles.resultGrid}>
        <div><span>REVISION</span><strong>12 → 13</strong></div>
        <div><span>EVENTS</span><strong>1 emitted</strong></div>
        <div><span>AUDIT</span><strong>recorded</strong></div>
      </div>
    </div>
  );
}

export default function Home(): ReactNode {
  return (
    <Layout
      title="The governed authority for engineering decisions"
      description="AEP Service gives humans and agents one attributable, transactional authority for engineering decisions across repositories.">
      <main>
        <header className={styles.hero}>
          <div className={styles.heroGrid} aria-hidden="true" />
          <div className={`container ${styles.heroInner}`}>
            <div className={styles.heroCopy}>
              <div className={styles.eyebrow}>
                <Status>PUBLIC DEVELOPER PREVIEW</Status>
                <span>v0.1</span>
              </div>
              <Heading as="h1">Engineering decisions,<br /><em>held as evidence.</em></Heading>
              <p className={styles.lede}>
                One central authority where people and agents submit semantic engineering intent—and
                every accepted change, refusal, revision, relation, and actor remains inspectable.
              </p>
              <div className={styles.actions}>
                <Link className={styles.primaryAction} to="/docs/quickstart">
                  Run the preview <span aria-hidden="true">→</span>
                </Link>
                <Link className={styles.secondaryAction} to="/docs/architecture">
                  See the architecture
                </Link>
              </div>
              <p className={styles.heroNote}>
                <span aria-hidden="true">◎</span> Local evaluation today. Production identity is deliberately not claimed.
              </p>
            </div>
            <AuthorityDiagram />
          </div>
        </header>

        <section className={styles.premise} aria-labelledby="premise-title">
          <div className={`container ${styles.premiseGrid}`}>
            <div>
              <p className={styles.sectionLabel}>THE MISSING AUTHORITY</p>
              <Heading as="h2" id="premise-title">Planning escaped the tracker.<br />Oversight did not follow.</Heading>
            </div>
            <div className={styles.premiseCopy}>
              <p>
                Agent-friendly planning moved into repositories because Markdown is close to code,
                versioned, and easy to operate. The cost is fragmented dependencies, stale references,
                and no complete view across repositories.
              </p>
              <p>
                AEP Service centralizes the entity record without taking the useful human view away.
                Markdown becomes a deterministic projection—not a competing source of truth.
              </p>
            </div>
          </div>
        </section>

        <section className={styles.flowSection} aria-labelledby="flow-title">
          <div className="container">
            <div className={styles.sectionHead}>
              <div>
                <p className={styles.sectionLabel}>FROM INTENT TO EVIDENCE</p>
                <Heading as="h2" id="flow-title">One door for every meaningful change.</Heading>
              </div>
              <p>The service composes released Engineering Protocols semantics with Entity Runtime storage. Neither dependency leaks through the public boundary.</p>
            </div>
            <ol className={styles.flowGrid}>
              {authorityFlow.map((step, index) => (
                <li key={step.number}>
                  <div className={styles.flowMeta}><span>{step.number}</span><b>{step.label}</b></div>
                  <Heading as="h3">{step.title}</Heading>
                  <p>{step.detail}</p>
                  {index < authorityFlow.length - 1 && <i aria-hidden="true">→</i>}
                </li>
              ))}
            </ol>
            <div className={styles.boundaryBand}>
              <span>PUBLIC CONTRACT</span>
              <p>Versioned AEP commands + bounded queries</p>
              <b aria-hidden="true">│</b>
              <span>PRIVATE MECHANISM</span>
              <p>Entity Runtime providers + PostgreSQL</p>
            </div>
          </div>
        </section>

        <section className={styles.traceSection} aria-labelledby="trace-title">
          <div className={`container ${styles.traceGrid}`}>
            <div className={styles.traceCopy}>
              <p className={styles.sectionLabel}>REFUSAL IS A RESULT</p>
              <Heading as="h2" id="trace-title">A “no” that explains itself is part of the record.</Heading>
              <p>
                Invalid transitions, stale revisions, missing capabilities, and unavailable authority
                do not disappear as exceptions in a client log. They return stable problem codes and
                safe details; eligible refusals become attributable audit facts.
              </p>
              <ul>
                <li><span>01</span>Authorization is decided before semantic materialization.</li>
                <li><span>02</span>A failed command publishes no prefix of candidate state.</li>
                <li><span>03</span>Retryability is explicit, not inferred from an HTTP number.</li>
              </ul>
              <Link className={styles.textAction} to="/docs/reliability">Read the reliability contract <span aria-hidden="true">↗</span></Link>
            </div>
            <div className={styles.problemPanel}>
              <div className={styles.panelBar}><span>409 / REVISION CONFLICT</span><b>NO MUTATION</b></div>
              <pre><code>{`{
  "request_id": "0198f03a…",
  "error": {
    "code": "revision_conflict",
    "message": "the entity changed",
    "retryable": false,
    "details": {
      "expected": 12,
      "actual": 13
    }
  }
}`}</code></pre>
              <div className={styles.problemFoot}><span>state</span><strong>unchanged</strong><span>audit</span><strong>attributed</strong></div>
            </div>
          </div>
        </section>

        <section className={styles.capabilitySection} aria-labelledby="capability-title">
          <div className="container">
            <div className={styles.sectionHead}>
              <div>
                <p className={styles.sectionLabel}>A NARROWER, STRONGER PROMISE</p>
                <Heading as="h2" id="capability-title">Not an issue tracker with an agent bolted on.</Heading>
              </div>
              <p>The service owns authority and evidence. Teams keep the review surfaces and integrations that fit their work.</p>
            </div>
            <div className={styles.capabilityGrid}>
              {capabilities.map((capability) => (
                <article key={capability.label}>
                  <span>{capability.label}</span>
                  <Heading as="h3">{capability.title}</Heading>
                  <p>{capability.text}</p>
                </article>
              ))}
            </div>
          </div>
        </section>

        <section className={styles.quickstartSection} aria-labelledby="quickstart-title">
          <div className={`container ${styles.quickstartGrid}`}>
            <div>
              <p className={styles.sectionLabel}>RUN THE RELEASED SURFACE</p>
              <Heading as="h2" id="quickstart-title">From empty database to governed story.</Heading>
              <p className={styles.quickstartLede}>Use the public multiarch image, PostgreSQL, and the pinned EP definitions. No local Rust toolchain is required.</p>
              <ol className={styles.quickSteps}>
                <li><span>1</span><div><strong>Pin the definitions</strong><p>Clone EP 0.38.1 and compute its immutable digest.</p></div></li>
                <li><span>2</span><div><strong>Start one authority</strong><p>Compose binds the preview listener to host loopback.</p></div></li>
                <li><span>3</span><div><strong>Submit intent</strong><p>Create, query, replay, then inspect history.</p></div></li>
              </ol>
              <Link className={styles.primaryAction} to="/docs/quickstart">Open the copy-paste quickstart <span aria-hidden="true">→</span></Link>
            </div>
            <div className={styles.terminal}>
              <div className={styles.terminalBar}><span><i /><i /><i /></span><b>preview / ready</b></div>
              <pre><code>{`$ docker compose -f compose.preview.yaml up -d

[+] postgres     healthy
[+] aep-service  healthy

$ curl -H "Authorization: Bearer $AEP_TOKEN" \\
    http://127.0.0.1:8080/readyz

HTTP/1.1 200 OK`}</code></pre>
              <div className={styles.terminalFoot}><span>linux/amd64</span><span>linux/arm64</span><strong>NON-ROOT · UID 10001</strong></div>
            </div>
          </div>
        </section>

        <section className={styles.statusSection} aria-labelledby="status-title">
          <div className="container">
            <div className={styles.sectionHead}>
              <div>
                <p className={styles.sectionLabel}>DEVELOPER PREVIEW</p>
                <Heading as="h2" id="status-title">Built boundaries, named gaps.</Heading>
              </div>
              <p>Public means inspectable. It does not mean the unfinished production trust boundary is quietly implied.</p>
            </div>
            <div className={styles.statusGrid}>
              <article>
                <div className={styles.statusHeading}><Status>AVAILABLE NOW</Status><span>{available.length} CAPABILITIES</span></div>
                <ul>{available.map((item) => <li key={item}><i aria-hidden="true">✓</i>{item}</li>)}</ul>
              </article>
              <article className={styles.nextCard}>
                <div className={styles.statusHeading}><span className={styles.nextLabel}>NEXT WAVES</span><span>{next.length} BOUNDARIES</span></div>
                <ul>{next.map((item) => <li key={item}><i aria-hidden="true">→</i>{item}</li>)}</ul>
              </article>
            </div>
          </div>
        </section>

        <section className={styles.finalCta}>
          <div className={`container ${styles.finalCtaInner}`}>
            <div>
              <p className={styles.sectionLabel}>START WITH THE EVIDENCE</p>
              <Heading as="h2">Evaluate the boundary.<br />Keep the claims honest.</Heading>
            </div>
            <div>
              <p>Run the local preview, inspect the derived contract, and tell us where a central engineering authority must be stronger.</p>
              <div className={styles.actions}>
                <Link className={styles.primaryAction} to="/docs/quickstart">Run the preview <span aria-hidden="true">→</span></Link>
                <Link className={styles.secondaryAction} to="/api">Explore the API</Link>
              </div>
            </div>
          </div>
        </section>
      </main>
    </Layout>
  );
}
