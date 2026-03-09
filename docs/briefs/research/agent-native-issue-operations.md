# Agent-Native Issue Operations

## Why this topic matters for Ishoo

Ishoo is explicitly trying to be a portable markdown issue tracker that works for both humans and AI agents. That means the issue format is not just a storage format. It is also an operating protocol. The question is not only "can a human read this?" but "can an agent safely create, edit, route, validate, and close work without destabilizing project memory?"

This dossier focuses on the best current understanding of agent-operable issue systems, especially where GitHub's public issue model and agent tooling conventions provide concrete guidance.

## The six research questions

1. What fields make an issue reliably operable by an AI agent?
2. How should dependency and hierarchy be modeled for agent workflows?
3. What parts of GitHub's issue model should Ishoo imitate directly?
4. What should labels do beyond categorization?
5. What machine-readable output surfaces matter most for CLI-first agents?
6. What guardrails are necessary if agents are allowed to author and mutate issue records?

## Question 1

### What fields make an issue reliably operable by an AI agent?

The strongest pattern in current issue tooling is that freeform prose is necessary but insufficient. GitHub issues, labels, dependencies, sub-issues, and automation APIs all assume a structured envelope around human text. For Ishoo, the implication is straightforward: agent-safe issues need a small, stable authored schema, not a large optional one.

The fields that matter most are the ones that directly change behavior:

- stable identity
- status
- labels
- dependency references
- explicit resolution field
- affected files or scope hints

GitHub's own product direction reinforces this. Issues support labels, relationships, and sub-issues because work routing and automation break down when everything is left in body prose alone. The GitHub Docs pages for issues, dependencies, labels, and sub-issues all point to the same principle: structured relationships are what make work queryable, automatable, and safe to evolve.

For Ishoo specifically, the current required trio of `Status`, `Labels`, and `Resolution` is directionally correct, but not yet the full agent envelope. The missing high-value field is scope. A `Files` field is extremely useful because it narrows search, sharpens retrieval for agents, and supports hotspot views. It should not necessarily be mandatory for every issue, but it is one of the best leverage points when present.

Another useful distinction from the literature and platform practice is between authored structure and derived structure. Labels, dependencies, and resolution are authored. Reverse links, transitive blockers, file heat, and section counts are derived. Ishoo should keep the authored schema compact and predictable, then derive everything else from it.

Inference from the sources: the best agent-facing format is not "maximally structured." It is "minimally sufficient and stable." Every authored field should either route work, constrain work, or explain closure.

## Question 2

### How should dependency and hierarchy be modeled for agent workflows?

GitHub now treats both dependencies and sub-issues as first-class relationships. Dependencies answer "what blocks this?" while sub-issues answer "what decomposes this?" Those are different semantics, and current platform design keeps them separate for a reason.

That distinction matters for Ishoo. If one markdown relation tries to do both jobs, agents will over-interpret it and users will under-maintain it. A dependency edge should mean completion order. A hierarchy edge should mean work breakdown. They are not interchangeable.

GitHub's dependency model is especially relevant because it improves board visibility and bottleneck detection without requiring deep project management ceremony. That maps well to Ishoo's philosophy. A simple `Depends on` list gives enough structure for blocking logic, next-up ranking, and graph views. Sub-issues are more optional, but the GitHub model shows they become valuable once the system supports deeper project decomposition.

The practical recommendation is:

- keep `Depends on` as the canonical blocking relation
- introduce parent-child work breakdown only when Ishoo is ready to support it consistently in CLI, UI, and lint
- do not overload labels or section placement to imply dependency or hierarchy

GitHub's sub-issue docs also show a useful ceiling: hierarchies can become deep, but complexity compounds quickly. Ishoo should resist large ontology design up front. A simple parent field or `Sub-issues` block would be enough when the product is ready.

Inference from the sources: for an agent-native tracker, dependency edges are higher priority than hierarchy because they directly affect execution order and automation. Hierarchy is valuable, but only once the product can represent it cleanly everywhere.

## Question 3

### What parts of GitHub's issue model should Ishoo imitate directly?

Ishoo should imitate the parts of GitHub's issue model that solve durable coordination problems, not the parts that exist because GitHub is a hosted multi-tenant service.

The most transferable features are:

- labels as a repository-level taxonomy
- dependencies as explicit blocking relationships
- sub-issues as optional decomposition
- autolinks and reference formats that stay readable in plain text

GitHub's autolink support is especially relevant because Ishoo lives in markdown and Git. A reference format becomes more valuable when it can survive outside the app: in commits, pull requests, chat, and markdown files. GitHub's documentation on autolinks and external autolinks points toward a broader principle: references should be short, stable, and easy to recognize by both humans and tools.

What Ishoo should not copy directly is GitHub's heavier project-management surface. Hosted assignment models, milestone mechanics, and repository permission assumptions do not map cleanly to a local-first markdown tracker. Ishoo's advantage is that it can stay lighter and more inspectable.

This suggests a useful standard for feature selection: if a GitHub issue feature improves portability, referenceability, or automation, it is a good candidate. If it depends on server authority or heavy process, it is less relevant.

Inference from the sources: Ishoo should aim for "GitHub-compatible mental models with lower ceremony." The closer it stays to familiar issue primitives, the easier it will be for humans and agents to reason about it without retraining.

## Question 4

### What should labels do beyond categorization?

GitHub's label docs and issue-labeler ecosystem make it clear that labels are not just tags for browsing. They are lightweight policy handles. They drive filtering, automation, queue shaping, and responsibility boundaries.

For Ishoo, that means labels should be treated as one of the main control surfaces of the system. A good label set should support:

- triage
- ownership domain
- workflow intent
- risk or urgency
- view filtering
- future automation

The current repo already uses labels for domains like `cli`, `feed`, `drag`, `viz`, and `docs`. That is useful because those labels map to code surface and likely work owners. But the research suggests a stronger governance rule: labels should represent stable axes, not temporary commentary. For example, "cli" is stable. "needs thinking" is usually not.

GitHub's default and reference label patterns also show the benefit of keeping taxonomies legible. A smaller, deliberate label set beats a sprawling folksonomy. Once a repository accumulates too many overlapping labels, both humans and agents start to route work inconsistently.

Inference from the sources: Ishoo should treat labels as required because they are the cheapest structured signal with the highest future automation value. It should also document a small set of approved label dimensions so the taxonomy does not drift into noise.

## Question 5

### What machine-readable output surfaces matter most for CLI-first agents?

If Ishoo wants the CLI to become the trusted automation surface, plain human-readable terminal output is not enough. GitHub's own issue APIs and CLI ecosystem demonstrate that automation works best when the same action can be consumed both by people and by tools.

For Ishoo, the highest-value machine-readable surfaces are:

- structured `show` output
- structured `list` output
- structured `lint` findings
- deterministic exit codes for all validation and mutation commands
- stable field names that mirror the markdown schema

The current `lint --strict` behavior is already on the right path because it uses exit code semantics correctly. The next step is to expose findings in JSON or line-delimited JSON so that agents and CI jobs do not have to parse prose output.

GitHub's labels and issue APIs reinforce the same lesson: once work becomes automation-facing, you need predictable fields and predictable failure modes. That is more important than flashy CLI formatting.

Inference from the sources: machine-readable output is not a nice-to-have for Ishoo. It is the difference between the CLI being a convenience layer and the CLI being the actual agent API.

## Question 6

### What guardrails are necessary if agents are allowed to author and mutate issue records?

The more autonomy agents get, the more the tracker must defend itself against silent schema drift and plausible-sounding garbage. Local markdown makes this risk worse because a malformed edit can still "look reasonable" in Git.

The strongest guardrails are the boring ones:

- required fields
- canonical status values
- dependency existence checks
- section coherence rules
- deterministic formatting
- explicit verification commands before close-out

That pattern aligns with both GitHub's issue model and the local repo's current direction. GitHub keeps labels repository-scoped, relationships explicit, and references syntactically constrained. Ishoo's lint rules are the local equivalent.

Another important guardrail is separation between authored and derived state. Agents should author issues. They should not hand-maintain counters, inferred backlinks, or aggregate views. That separation reduces error surface dramatically.

Inference from the sources: the safest agent model is not "trust the agent less," but "give the agent fewer ambiguous degrees of freedom." Ishoo should keep reducing optional interpretation in the issue format.

## Topic Synthesis for Ishoo

The current state of the art points toward a clear product direction.

First, Ishoo should continue treating issue markdown as a protocol, not just a note format. The winning pattern is a compact authored schema with strong linting, not a loose markdown blob plus heuristics.

Second, dependencies deserve continued first-class investment because they are the highest-value relationship for agent execution. Hierarchy is next, but only once it can be represented consistently in CLI, UI, and persistence.

Third, labels should be treated as governance and automation handles, not merely tags. That implies required presence, a documented taxonomy, and eventual machine-readable CLI output that surfaces labels cleanly.

Fourth, CLI parity and machine-readable output should be considered core product work, not polish. If Ishoo wants to be terminal-first and agent-native, JSON output and stable exit codes are not optional.

The most important near-term implications for the roadmap are:

- make `Files` stronger as a scoped field, even if not always mandatory
- add machine-readable output modes for CLI commands
- preserve explicit dependency semantics and resist overloading labels
- design future hierarchy support as a separate concept from blocking
- keep lint strict and schema-oriented

## Sources

- GitHub Docs: [About issues](https://docs.github.com/articles/about-issues)
- GitHub Docs: [Creating issue dependencies](https://docs.github.com/en/enterprise-cloud%40latest/issues/tracking-your-work-with-issues/using-issues/creating-issue-dependencies)
- GitHub Docs: [Adding sub-issues](https://docs.github.com/en/enterprise-server%403.20/issues/tracking-your-work-with-issues/using-issues/adding-sub-issues)
- GitHub Docs: [Managing labels](https://docs.github.com/articles/editing-a-label)
- GitHub Docs: [Configuring autolinks to reference external resources](https://docs.github.com/articles/configuring-autolinks-to-reference-external-resources)
- GitHub Docs: [Autolinked references and URLs](https://docs.github.com/en/enterprise-server%403.14/get-started/writing-on-github/working-with-advanced-formatting/autolinked-references-and-urls)
- GitHub Docs: [REST API endpoints for labels](https://docs.github.com/rest/issues/labels)

