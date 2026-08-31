import type {ReactNode} from 'react';
import {useMemo, useState} from 'react';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import specificationJson from '../../static/openapi.json';

import styles from './api.module.css';

type Schema = {
  $ref?: string;
  type?: string | string[];
  format?: string;
  description?: string;
  enum?: unknown[];
  const?: unknown;
  properties?: Record<string, Schema>;
  required?: string[];
  items?: Schema;
  anyOf?: Schema[];
  oneOf?: Schema[];
  allOf?: Schema[];
  additionalProperties?: boolean | Schema;
  minimum?: number;
  maximum?: number;
  minLength?: number;
  maxLength?: number;
};

type Example = {summary?: string; value?: unknown};
type Media = {schema?: Schema; examples?: Record<string, Example>};
type Parameter = {name: string; in: string; required?: boolean; description?: string; schema?: Schema};
type Response = {description?: string; content?: Record<string, Media>};
type Operation = {
  operationId: string;
  summary: string;
  description?: string;
  tags?: string[];
  parameters?: Parameter[];
  requestBody?: {required?: boolean; content?: Record<string, Media>};
  responses?: Record<string, Response>;
};
type OpenApi = {
  info: {title: string; version: string; description?: string};
  paths: Record<string, Record<string, Operation>>;
  components: {schemas: Record<string, Schema>};
};
type ProjectedOperation = Operation & {method: string; path: string; tag: string};

const specification = specificationJson as unknown as OpenApi;
const methods = new Set(['get', 'post', 'put', 'patch', 'delete']);
const operations: ProjectedOperation[] = Object.entries(specification.paths).flatMap(([path, item]) =>
  Object.entries(item)
    .filter(([method]) => methods.has(method))
    .map(([method, operation]) => ({
      ...operation,
      method: method.toUpperCase(),
      path,
      tag: operation.tags?.[0] ?? 'Other',
    })),
);
const groups = [...new Set(operations.map((operation) => operation.tag))];
const schemas = specification.components.schemas;

function referenceName(schema?: Schema): string | undefined {
  return schema?.$ref?.split('/').at(-1);
}

function typeLabel(schema?: Schema): string {
  if (!schema) return 'document';
  const reference = referenceName(schema);
  if (reference) return reference;
  if (schema.enum) return schema.enum.map(String).join(' | ');
  if (schema.const !== undefined) return JSON.stringify(schema.const);
  if (schema.anyOf) return schema.anyOf.map(typeLabel).join(' | ');
  if (schema.oneOf) return schema.oneOf.map(typeLabel).join(' | ');
  if (schema.allOf) return schema.allOf.map(typeLabel).join(' & ');
  const type = Array.isArray(schema.type) ? schema.type.join(' | ') : schema.type;
  if (type === 'array') return `${typeLabel(schema.items)}[]`;
  return [type ?? 'object', schema.format].filter(Boolean).join(' · ');
}

function schemaLink(schema?: Schema): ReactNode {
  const reference = referenceName(schema);
  if (!reference) return <code>{typeLabel(schema)}</code>;
  return <a href={`#schema-${reference}`}><code>{reference}</code></a>;
}

function firstMedia(content?: Record<string, Media>): [string, Media] | undefined {
  return content ? Object.entries(content)[0] : undefined;
}

function firstExample(media?: Media): Example | undefined {
  return media?.examples ? Object.values(media.examples)[0] : undefined;
}

function routeForCurl(path: string): string {
  return path
    .replace('{realm}', 'demo')
    .replace('{workspace}', 'plan')
    .replace('{entity}', '$AEP_ENTITY_ID')
    .replace('{entity_type}', 'aep.story%2Fv1');
}

function curlExample(operation: ProjectedOperation): string {
  const requestMedia = firstMedia(operation.requestBody?.content);
  const responseMedia = firstMedia(operation.responses?.['200']?.content);
  const mediaType = requestMedia?.[0] ?? responseMedia?.[0] ?? 'application/vnd.aep.service+json;version=1';
  const body = firstExample(requestMedia?.[1])?.value;
  const lines = [
    `curl --request ${operation.method} \\`,
    '  --header "Authorization: Bearer $AEP_DEV_BEARER_TOKEN" \\',
    `  --header "Accept: ${mediaType}" \\`,
  ];
  if (body !== undefined) {
    lines.push(`  --header "Content-Type: ${mediaType}" \\`);
    lines.push(`  --data '${JSON.stringify(body, null, 2)}' \\`);
  }
  lines.push(`  "http://127.0.0.1:8080${routeForCurl(operation.path)}"`);
  return lines.join('\n');
}

function CopyButton({value, label = 'Copy'}: {value: string; label?: string}): ReactNode {
  const [copied, setCopied] = useState(false);
  async function copy(): Promise<void> {
    await navigator.clipboard.writeText(value);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1500);
  }
  return <button className={styles.copyButton} type="button" onClick={copy}>{copied ? 'Copied' : label}</button>;
}

function CodeExample({value, label}: {value: unknown; label: string}): ReactNode {
  const rendered = typeof value === 'string' ? value : JSON.stringify(value, null, 2);
  return (
    <div className={styles.codeExample}>
      <div><span>{label}</span><CopyButton value={rendered} /></div>
      <pre><code>{rendered}</code></pre>
    </div>
  );
}

function Parameters({parameters}: {parameters?: Parameter[]}): ReactNode {
  if (!parameters?.length) return null;
  return (
    <div className={styles.parameterBlock}>
      <h4>Path parameters</h4>
      <dl>
        {parameters.map((parameter) => (
          <div key={parameter.name}>
            <dt><code>{parameter.name}</code><span>{typeLabel(parameter.schema)}</span></dt>
            <dd>{parameter.description}</dd>
          </div>
        ))}
      </dl>
    </div>
  );
}

function Responses({responses}: {responses?: Record<string, Response>}): ReactNode {
  if (!responses) return null;
  return (
    <div className={styles.responseBlock}>
      <h4>Responses</h4>
      <div className={styles.responseList}>
        {Object.entries(responses).map(([status, response]) => {
          const media = firstMedia(response.content);
          return (
            <div key={status}>
              <strong className={`${styles.statusCode} ${status.startsWith('2') ? styles.statusOk : ''}`}>{status}</strong>
              <p>{response.description}</p>
              {schemaLink(media?.[1].schema)}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function OperationCard({operation}: {operation: ProjectedOperation}): ReactNode {
  const requestMedia = firstMedia(operation.requestBody?.content);
  const requestExample = firstExample(requestMedia?.[1]);
  const successMedia = firstMedia(operation.responses?.['200']?.content);
  const successExample = firstExample(successMedia?.[1]);
  const problemMedia = firstMedia(operation.responses?.['409']?.content);
  const problemExample = firstExample(problemMedia?.[1]);
  return (
    <article className={styles.operation} id={operation.operationId}>
      <div className={styles.operationHead}>
        <div className={styles.operationRoute}>
          <span className={`${styles.method} ${styles[`method${operation.method}`]}`}>{operation.method}</span>
          <code>{operation.path}</code>
        </div>
        <span className={styles.operationGroup}>{operation.tag}</span>
      </div>
      <div className={styles.operationBody}>
        <div className={styles.operationIntro}>
          <div>
            <span className={styles.operationId}>{operation.operationId}</span>
            <Heading as="h2"><a href={`#${operation.operationId}`}>{operation.summary}</a></Heading>
            <p>{operation.description}</p>
          </div>
          <CodeExample value={curlExample(operation)} label="curl" />
        </div>
        <div className={styles.contractGrid}>
          <div>
            <Parameters parameters={operation.parameters} />
            <div className={styles.requestBlock}>
              <h4>Request</h4>
              {requestMedia ? (
                <>
                  <p className={styles.mediaType}>{requestMedia[0]}</p>
                  <p>Strict body: {schemaLink(requestMedia[1].schema)}</p>
                </>
              ) : <p>No request body.</p>}
            </div>
          </div>
          <Responses responses={operation.responses} />
        </div>
        {(requestExample || successExample || problemExample) && (
          <div className={styles.examples}>
            {requestExample && <CodeExample value={requestExample.value} label={requestExample.summary ?? 'Request'} />}
            {successExample && <CodeExample value={successExample.value} label={successExample.summary ?? 'Success'} />}
            {problemExample && <CodeExample value={problemExample.value} label={problemExample.summary ?? 'Problem'} />}
          </div>
        )}
      </div>
    </article>
  );
}

function SchemaShape({schema, depth = 0}: {schema: Schema; depth?: number}): ReactNode {
  const reference = referenceName(schema);
  if (reference) return <a href={`#schema-${reference}`}><code>{reference}</code></a>;
  const composition = schema.anyOf ?? schema.oneOf ?? schema.allOf;
  if (composition) {
    return <span className={styles.composed}>{composition.map((part, index) => <span key={index}>{schemaLink(part)}</span>)}</span>;
  }
  if (!schema.properties || depth >= 2) return <code>{typeLabel(schema)}</code>;
  return (
    <dl className={styles.schemaProperties}>
      {Object.entries(schema.properties).map(([name, property]) => (
        <div key={name}>
          <dt><code>{name}</code>{schema.required?.includes(name) && <b>required</b>}<span>{typeLabel(property)}</span></dt>
          {property.description && <dd>{property.description}</dd>}
          {!property.$ref && property.properties && <dd><SchemaShape schema={property} depth={depth + 1} /></dd>}
        </div>
      ))}
    </dl>
  );
}

function SchemaCatalog(): ReactNode {
  return (
    <section className={styles.schemaSection} id="schemas" aria-labelledby="schemas-title">
      <div className={styles.sectionHead}>
        <div><span>COMPONENT CATALOG</span><Heading as="h2" id="schemas-title">Strict wire schemas</Heading></div>
        <p>Generated from the released EP DTOs. Unknown fields are refused; nullable members remain required where omission would change wire meaning.</p>
      </div>
      <div className={styles.schemaGrid}>
        {Object.entries(schemas).map(([name, schema]) => (
          <details id={`schema-${name}`} key={name}>
            <summary><code>{name}</code><span>{typeLabel(schema)}</span></summary>
            <div className={styles.schemaBody}>
              {schema.description && <p>{schema.description}</p>}
              <SchemaShape schema={schema} />
            </div>
          </details>
        ))}
      </div>
    </section>
  );
}

export default function Api(): ReactNode {
  const specificationUrl = useBaseUrl('/openapi.json');
  const [query, setQuery] = useState('');
  const [group, setGroup] = useState('All');
  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return operations.filter((operation) => {
      const inGroup = group === 'All' || operation.tag === group;
      const haystack = `${operation.operationId} ${operation.summary} ${operation.description} ${operation.path} ${operation.tag}`.toLowerCase();
      return inGroup && (!needle || haystack.includes(needle));
    });
  }, [group, query]);

  return (
    <Layout title="API reference" description="Static reference for the derived AEP Service HTTP contract">
      <main className={styles.apiPage}>
        <header className={styles.hero}>
          <div className="container">
            <p className={styles.eyebrow}>DERIVED CONTRACT / OPENAPI 3.1</p>
            <div className={styles.heroGrid}>
              <div>
                <Heading as="h1">A semantic API,<br /><em>not a storage API.</em></Heading>
                <p>{specification.info.description}</p>
                <div className={styles.heroActions}>
                  <a className={styles.primaryAction} href={specificationUrl} download>Download OpenAPI JSON</a>
                  <Link className={styles.secondaryAction} to="/docs/http-contract">Read the HTTP contract</Link>
                </div>
              </div>
              <dl className={styles.apiFacts}>
                <div><dt>VERSION</dt><dd>{specification.info.version}</dd></div>
                <div><dt>OPERATIONS</dt><dd>{operations.length}</dd></div>
                <div><dt>SCHEMAS</dt><dd>{Object.keys(schemas).length}</dd></div>
                <div><dt>AUTH</dt><dd>Bearer</dd></div>
              </dl>
            </div>
          </div>
        </header>

        <section className={`container ${styles.reference}`}>
          <div className={styles.referenceIntro}>
            <div>
              <p className={styles.sectionLabel}>OPERATIONS</p>
              <Heading as="h2">Commands and bounded questions.</Heading>
            </div>
            <p>All content below is rendered from the same generated document served by the process. Search and copy controls enhance the page; the reference itself does not depend on hydration.</p>
          </div>
          <div className={styles.filters}>
            <label><span>Filter operations</span><input type="search" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search path, operation, or concept" /></label>
            <div className={styles.groupFilters} aria-label="Filter by operation group">
              {['All', ...groups].map((candidate) => <button type="button" className={group === candidate ? styles.activeFilter : ''} onClick={() => setGroup(candidate)} key={candidate}>{candidate}</button>)}
            </div>
          </div>
          <p className={styles.resultCount}>{filtered.length} of {operations.length} operations</p>
          <div className={styles.operationList}>
            {filtered.map((operation) => <OperationCard operation={operation} key={operation.operationId} />)}
          </div>
          {filtered.length === 0 && <div className={styles.empty}>No operation matches this filter.</div>}
          <SchemaCatalog />
        </section>
      </main>
    </Layout>
  );
}
