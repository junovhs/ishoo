# Local-First and Git-Backed Architecture

## Why this topic matters for Ishoo

Ishoo's core promise is that issues live as plain markdown files in the repo. That puts it in the local-first family, but in a very specific variant: Git-backed, text-first, and currently single-user or loosely coordinated rather than real-time multiplayer.

This dossier looks at what current local-first research, Git primitives, and conflict-management tooling imply for Ishoo's architecture trajectory.

## The six research questions

1. What does the local-first literature say that is directly applicable to Ishoo?
2. When is plain markdown plus Git enough, and when do you need a CRDT or sync engine?
3. What does Git already offer for structured-text merge management?
4. What identity model best fits a portable markdown issue tracker?
5. How should history and branching be treated as product features rather than implementation details?
6. What is a plausible long-term architecture path if Ishoo later adds collaboration?

## Question 1

### What does the local-first literature say that is directly applicable to Ishoo?

The Ink and Switch local-first essay remains the most relevant framing. Its central claim is that software should preserve user ownership and local agency while still supporting the benefits people associate with cloud systems. That aligns almost exactly with Ishoo's philosophy.

The most applicable local-first properties for Ishoo are:

- the user owns the data
- the data remains usable without the app
- offline operation is natural, not degraded
- history and branching are first-class
- the system should degrade gracefully when vendors disappear

Ishoo already gets several of these almost for free by storing plain markdown in Git. The user can inspect the data without the app, branch it with standard Git tooling, and preserve it long-term. That is a strong architectural position.

What Ishoo does not yet fully realize is the "best of both worlds" part of local-first. The current model is excellent at durability and inspectability, but weaker at merge friendliness, multi-user semantics, and structured machine interpretation. The local-first literature suggests that is normal. Ownership is the first win; smooth collaboration is the harder second win.

Inference from the sources: Ishoo is correctly positioned as a local-first tool already. The next architectural work should focus less on storage format debates and more on conflict semantics, machine-owned identity, and collaboration boundaries.

## Question 2

### When is plain markdown plus Git enough, and when do you need a CRDT or sync engine?

Automerge and related local-first systems show what CRDT-based sync engines are good at: concurrent editing, offline-first replication, and deterministic merges across devices and users. But that does not mean every local-first tool should adopt a CRDT.

For Ishoo, plain markdown plus Git is still enough if these assumptions hold:

- editing is mostly asynchronous
- conflicts are relatively rare
- human review of diffs matters
- repo portability is a primary product value
- the system does not require true multi-user live editing

Those assumptions fit the current app well. Ishoo is much closer to "version-controlled project memory" than to "Google Docs for issues."

CRDT infrastructure becomes more relevant only if Ishoo later needs:

- real-time multi-user editing
- automatic resolution of fine-grained concurrent edits
- frequent disconnected multi-device editing without Git mediation

The Peritext work is especially useful here. It demonstrates that rich, intent-preserving merge semantics are hard even for text. That is a warning, not an invitation. Ishoo should not jump to CRDT complexity unless the product truly needs that class of collaboration.

Inference from the sources: Ishoo should not chase a CRDT architecture yet. The better path is to harden markdown plus Git first, because that is where the product's differentiator lives.

## Question 3

### What does Git already offer for structured-text merge management?

Git already provides more leverage here than many local tools use. The `gitattributes` documentation is particularly relevant because it shows three important levers:

- per-path merge strategy selection
- custom merge drivers
- `merge.renormalize` for canonical-format transitions

For plain markdown issue files, Git's default text merge is often good enough. But once issue files become more structurally important, a custom merge driver becomes interesting. Git explicitly supports file-specific merge drivers that receive ancestor, local, and remote versions and write back the merged result. That creates a clear future path for Ishoo: parse the issue markdown structurally, merge at the issue-record level, and emit canonical markdown again.

The built-in `union` merge driver is also informative because it shows what not to do casually. It can combine both sides without conflict markers, but order becomes unreliable. That is risky for structured issue records because record order and block boundaries may matter.

Inference from the sources: the best medium-term merge strategy for Ishoo is not to abandon Git, but to layer a structural merge driver on top of Git's existing merge model when conflict frequency justifies it.

## Question 4

### What identity model best fits a portable markdown issue tracker?

The repo's open issue about machine-owned identity is well aligned with the broader architecture evidence. Human-authored IDs are convenient for discussion, but brittle as system identifiers.

Local-first systems and sync engines generally separate stable internal identity from user-facing representation. Automerge's model is a good reminder that durable identity and readable labels do not have to be the same thing. GitHub also separates database identity from human issue numbers, even if users mostly see the latter.

For Ishoo, the right long-term shape looks like this:

- machine-owned stable internal identity
- human-visible short handle for discussion and navigation
- references and dependencies attached to stable identity, not mere display order

That does not require hiding visible IDs. It only requires ceasing to treat visible IDs as the canonical source of truth.

Inference from the sources: the product should preserve readable handles while moving system correctness onto a machine-owned identity layer. That is the architecture that scales to migration, renumbering, and richer automation.

## Question 5

### How should history and branching be treated as product features rather than implementation details?

Automerge markets versioning as a core property, and Git obviously makes branching central. Ishoo should learn from that. History is not just a storage accident. It is part of the product.

For users, the most valuable history features are not full VCS complexity inside the UI. They are narrow, practical capabilities:

- issue age
- last modified information
- change summaries
- branch-aware status or warning surfaces
- visibility into divergent edits

The active issue about project health and issue age points in the right direction. If Ishoo is Git-backed, then Git-derived metadata should eventually become part of the user experience. That would strengthen the product's "stealth tracker" identity: the issue system is native to the repo rather than adjacent to it.

Inference from the sources: Ishoo should expose Git-derived history selectively and intentionally. It should not try to become a Git client, but it should absolutely become Git-aware.

## Question 6

### What is a plausible long-term architecture path if Ishoo later adds collaboration?

The current evidence suggests a staged path rather than a big-bang redesign.

Stage 1:

- harden markdown schema
- introduce machine-owned identity
- improve CLI automation
- reduce merge pain with lint and deterministic formatting

Stage 2:

- add structural merge support for issue files
- expose Git-derived change intelligence in UI and CLI
- support safer branching and sync workflows

Stage 3, only if product demand exists:

- add optional sync metadata or CRDT-backed internal editing model
- preserve markdown export as the durable artifact
- keep a strong escape hatch back to text and Git

Automerge is relevant mostly as a future option, not a present prescription. It shows that offline-capable replicated data structures can work at scale. But Ishoo's differentiator is that the durable source of truth is inspectable text in a normal repo. Any collaboration upgrade should preserve that.

Inference from the sources: if Ishoo ever adopts collaborative sync tech, it should be additive and reversible. Markdown and Git should remain the final artifact layer.

## Topic Synthesis for Ishoo

The state of the art strongly validates Ishoo's core architecture. Local-first is not a trend label here; it is an accurate description of the product's deepest strength.

The clearest near-term conclusion is that Ishoo should keep investing in markdown plus Git rather than trying to outgrow them too early. The right next steps are structural correctness and conflict resilience:

- machine-owned identity
- deterministic formatting
- possible future structural merge support
- selective Git-aware UI and CLI features

The main strategic warning is that collaboration complexity is real. The local-first literature and CRDT ecosystem show that rich merging semantics are expensive. Ishoo should only adopt that complexity in response to real product pressure.

The best roadmap implication is therefore conservative but strong:

- treat Git as a feature, not just a backend
- keep markdown as the durable artifact
- improve semantics before adding infrastructure

## Sources

- Ink and Switch: [Local-first software: You own your data, in spite of the cloud](https://www.inkandswitch.com/essay/local-first/)
- Automerge: [Automerge](https://automerge.org/)
- Ink and Switch: [Peritext](https://www.inkandswitch.com/project/peritext/)
- Git documentation: [gitattributes](https://git-scm.com/docs/gitattributes/2.50.0.html)

