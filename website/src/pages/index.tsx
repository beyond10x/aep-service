import React from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';

export default function Home(): React.JSX.Element {
  return (
    <Layout title="Governed engineering decisions" description="A central AEP authority for humans and agents.">
      <header className="hero hero--primary">
        <div className="container">
          <h1 className="hero__title">AEP Service</h1>
          <p className="hero__subtitle">One attributable, transactional authority for engineering decisions across repositories.</p>
          <div>
            <Link className="button button--secondary button--lg" to="/docs/quickstart">Run the developer preview</Link>{' '}
            <Link className="button button--outline button--secondary button--lg" to="/api">Explore the API</Link>
          </div>
        </div>
      </header>
      <main className="container margin-vert--xl">
        <div className="row">
          <section className="col col--4"><h2>Semantic, not generic</h2><p>Clients submit AEP commands and queries. SQL, provider batches, and raw entity-store operations stay private.</p></section>
          <section className="col col--4"><h2>Humans and agents</h2><p>Trusted authority and executor are separate facts. Delegation is designed to narrow an owner’s grants, never expand them.</p></section>
          <section className="col col--4"><h2>Evidence by default</h2><p>State, history, relations, events, idempotency, and refusals commit through one PostgreSQL transaction.</p></section>
        </div>
        <div className="alert alert--warning margin-top--lg"><strong>Developer preview:</strong> the current bearer verifier is local-development infrastructure, not production SSO.</div>
      </main>
    </Layout>
  );
}
