import React, {useEffect, useState} from 'react';
import Layout from '@theme/Layout';
import useBaseUrl from '@docusaurus/useBaseUrl';

type Operation = {method: string; path: string; id: string; summary: string};

export default function Api(): React.JSX.Element {
  const specification = useBaseUrl('/openapi.json');
  const [operations, setOperations] = useState<Operation[]>([]);
  const [failure, setFailure] = useState<string>();
  useEffect(() => {
    fetch(specification)
      .then((response) => { if (!response.ok) throw new Error(`${response.status}`); return response.json(); })
      .then((document) => {
        const projected: Operation[] = [];
        for (const [path, item] of Object.entries<Record<string, Record<string, string>>>(document.paths ?? {})) {
          for (const [method, operation] of Object.entries<any>(item)) {
            projected.push({method: method.toUpperCase(), path, id: operation.operationId, summary: operation.summary});
          }
        }
        setOperations(projected);
      })
      .catch((error) => setFailure(String(error)));
  }, [specification]);
  return (
    <Layout title="API reference" description="Derived AEP Service HTTP operations">
      <main className="container margin-vert--lg">
        <h1>API reference</h1>
        <p>This page reads the OpenAPI document generated from Engineering Protocols’ strict wire DTOs and route catalog. <a href={specification} download>Download the complete specification</a>.</p>
        {failure && <div className="alert alert--danger">Could not load the generated specification: {failure}</div>}
        <div className="api-grid">
          {operations.map((operation) => <article className="api-card" key={operation.id}>
            <div><span className="api-method">{operation.method}</span><code>{operation.path}</code></div>
            <h3>{operation.summary}</h3><small>{operation.id}</small>
          </article>)}
        </div>
      </main>
    </Layout>
  );
}
