# Docs Writing Style

Use this guide when editing product docs in `apps/docs/content/docs`.

The goal is simple: write docs that sound like a careful engineer, not a marketing page and not a polished text generator.

## Core Rules

1. Lead with the concrete point.
2. Prefer plain verbs over abstract nouns.
3. State what changes, where to look, or what to do next.
4. Cut filler that only makes the text sound smoother.
5. Keep paragraphs short, but do not stack three vague one-line paragraphs just for rhythm.

## Patterns To Avoid

- Inflated framing such as "powerful", "seamless", "robust", "transformative", or "comprehensive" unless the page proves the claim.
- Abstract lead-ins such as "Qryvanta provides a framework for..." when you can say what the user does instead.
- Repetitive list scaffolds where every section uses the same "Start here / then / finally" phrasing without adding meaning.
- Significance inflation where ordinary behavior is described as strategic, critical, or foundational without a real distinction.
- Decorative transitions and throat-clearing such as "In today's landscape", "it is important to note", or "this serves as".
- Em dashes used as a style tic. Use a period most of the time.

## Preferred Patterns

- Name the actor: "Admin Center controls roles" is better than "Role management is handled in Admin Center."
- Name the boundary: draft vs published, Maker vs Worker, API vs worker, local vs self-hosted.
- Tell the reader what to check first when something fails.
- Use lists for real procedures, prerequisites, and checks. Use prose for explanation.
- Add a short summary block when a page benefits from "use this when", "you need", or "common mistake" guidance.

## Editing Checklist

Before you finish a docs change, check these points:

1. The first paragraph says what the page is for in plain language.
2. Important commands, routes, config names, and product surfaces are explicit.
3. Every list item adds distinct information rather than repeating the same sentence pattern.
4. If a sentence sounds polished but vague, replace it with a concrete statement or delete it.
5. The page tells the reader what to read or check next when that would help.

## Example Rewrites

- Instead of: "Qryvanta delivers a comprehensive metadata-driven experience for modern teams."
- Write: "Qryvanta lets teams define entities, forms, views, and workflows in metadata."

- Instead of: "This page provides guidance for navigating the operational landscape."
- Write: "Use this page when you are deploying Qryvanta outside local development."

- Instead of: "Published metadata acts as a critical foundation for runtime experiences."
- Write: "Worker Apps uses the published metadata model."
