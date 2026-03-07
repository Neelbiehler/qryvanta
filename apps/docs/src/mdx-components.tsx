import defaultMdxComponents from 'fumadocs-ui/mdx';
import type { MDXComponents } from 'mdx/types';
import {
  AudienceTag,
  AudienceTags,
  Checklist,
  DocCallout,
  DocCard,
  DocCardGrid,
  DocSummary,
  DocSummaryItem,
} from '@/components/docs/primitives';

export function getMDXComponents(components?: MDXComponents): MDXComponents {
  return {
    ...defaultMdxComponents,
    AudienceTag,
    AudienceTags,
    Checklist,
    DocCallout,
    DocCard,
    DocCardGrid,
    DocSummary,
    DocSummaryItem,
    ...components,
  };
}
