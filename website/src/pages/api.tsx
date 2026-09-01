import type {ReactNode} from 'react';
import Layout from '@theme/Layout';
import useBaseUrl from '@docusaurus/useBaseUrl';
import {OpenApiReference, type OpenApiDocument} from '@beyond10x/docs-system/renderers';
import specificationJson from '../../static/openapi.json';

/** The service owns the OpenAPI bytes; docs-system owns their reusable presentation. */
export default function ApiReference(): ReactNode {
  const sourceUrl = useBaseUrl('/openapi.json');
  return (
    <Layout
      title="API reference"
      description="Read-only OpenAPI 3.1 reference for AEP Service commands and queries.">
      <OpenApiReference
        document={specificationJson as unknown as OpenApiDocument}
        sourceUrl={sourceUrl}
      />
    </Layout>
  );
}
