# Desktop Interaction, Accessibility, and Visualization

## Why this topic matters for Ishoo

The repo's active issues make the current pressure obvious: drag feel, board/feed parity, scroll smoothness, verification discipline, and better visualization all matter. Ishoo is not just a parser and CLI. It is also a desktop dashboard where motion quality and information density affect whether the tool feels premium or brittle.

This dossier focuses on interaction quality, accessible control patterns, and graph or hotspot visualization choices that map directly to the app's UI surface.

## The six research questions

1. What does current interaction research imply about drag-and-drop quality?
2. What accessibility model should guide reorderable task lists and boards?
3. What Dioxus-specific reactivity lessons matter for motion-heavy surfaces?
4. When should Ishoo use node-link graphs versus matrix-like views?
5. What do network-visualization studies suggest for dependency and overlap views?
6. How should hotspot and progress visualization avoid becoming noise?

## Question 1

### What does current interaction research imply about drag-and-drop quality?

The main lesson from Fitts' Law research is not "make targets larger" in a generic way. The more precise lesson is that movement tasks must be modeled as the task users are actually performing. Dragging is not equivalent to pointing, and low-speed controlled movement has its own difficulty profile.

The classic Gillan et al. result is useful here because it specifically distinguishes point-click from point-drag behavior. For Ishoo, that means slow drag quality should be treated as a separate interaction problem, not just a frame-rate variant of clicking or flicking. The open issue about "locked-solid slow drag" is well posed for exactly that reason.

Two practical consequences follow:

- visual stability at low speed matters more than raw peak speed
- effective target width during drag depends on motion direction and task geometry

That supports several design instincts already visible in the repo: local displacement instead of long teleports, stable cursor-card locking, and minimizing visual shimmer during drag. The research does not prescribe exact animation parameters, but it strongly supports task-specific tuning rather than generic "smoothness" claims.

Inference from the sources: Ishoo should verify drag quality with slow, deliberate reorder tasks as a first-class benchmark, because that is where the interaction difficulty is most exposed.

## Question 2

### What accessibility model should guide reorderable task lists and boards?

The WAI-ARIA Authoring Practices grid pattern is the most relevant official model in the current source set. It is not a drag-and-drop tutorial, but it provides the keyboard-navigation logic for dense, interactive collections where only one item should sit in the tab order at a time.

That maps well to Ishoo's feed and board. Both are collections of interactive cards, and both already risk becoming hard to navigate if every element becomes a tab stop. The APG grid guidance suggests a solid accessible shape:

- roving focus within the collection
- arrow-key navigation
- explicit keyboard move operations for reorderable items
- reduced tab-stop burden for dense views

The deprecated state of `aria-dropeffect` is also useful because it shows that Ishoo should not rely on old drag-specific ARIA semantics as the accessibility story. The safer approach is keyboard-operable reorder commands plus live feedback and clear focus management.

Inference from the sources: Ishoo's accessible reordering story should be "keyboard-first reorder with strong announcements," not "ARIA drag attributes will solve it."

## Question 3

### What Dioxus-specific reactivity lessons matter for motion-heavy surfaces?

The Dioxus signal documentation is unusually relevant to the current drag issues. Signals use local subscriptions: components only rerender when they read the signal in their reactive scope. That is powerful, but it also means careless signal reads can broaden rerender blast radius in motion-heavy paths.

The docs around `Signal`, `use_signal`, `use_hook`, and reactive contexts point to a concrete engineering rule: in a drag pipeline, isolate reactive reads as tightly as possible. Event handlers can write without subscribing the same way render scopes do, and `peek` exists specifically to avoid accidental reactive coupling when reading and writing from the same signal ecosystem.

For Ishoo, that reinforces the likely direction of the open drag-performance work:

- reduce parent and sibling reactive reads during active drag
- keep drag-state ownership local
- avoid derived UI churn that marks large scopes dirty every frame
- prefer narrow subscriptions and computed state where they truly cut rerender spread

Inference from the sources: the likely performance ceiling is governed less by "Rust vs JS" and more by reactive scope hygiene. Dioxus gives the right primitives, but the app has to use them with discipline in the hot path.

## Question 4

### When should Ishoo use node-link graphs versus matrix-like views?

The modern visualization evidence is fairly consistent. Node-link diagrams are intuitive for path tracing and small sparse graphs. Adjacency matrices are more reliable as graphs become denser or when accuracy across tasks matters more than visual familiarity. Recent comparison work adds a third option, bipartite layouts, which can be useful for revealing overall structure in large directed networks.

For Ishoo, the implication is not that the current node-link view is wrong. It is that it should not be the only graph idiom. Issue dependencies and file-overlap graphs can become visually cluttered fast, especially once labels, sections, and overlap strength are added.

The practical rule should be:

- use node-link for small dependency neighborhoods and path-oriented exploration
- use matrix or matrix-like summaries for dense overlap analysis
- consider bipartite or linked hybrid views when connecting issues to files

The dynamic linked graph-and-matrix literature is especially relevant because Ishoo already has both relational and tabular questions to answer. Users sometimes want to trace a path. Other times they want reliable overview and comparison.

Inference from the sources: Ishoo should think in terms of complementary graph views, not a single all-purpose dependency diagram.

## Question 5

### What do network-visualization studies suggest for dependency and overlap views?

The recent comparisons between node-link, matrix, and bipartite representations produce a useful design pattern. Different tasks favor different representations:

- overview and density estimation often benefit from matrix or bipartite forms
- path following and local neighborhood tracing often favor node-link
- large dense graphs become unreliable if forced into node-link alone

That suggests Ishoo's overlap and dependency views should be task-sliced. A graph view can stay as the exploratory surface. But the app should eventually complement it with one or both of:

- a dense overlap matrix for file or issue intersections
- a bipartite issue-to-file view for structure discovery

This would align especially well with Ishoo's "hotspot" and "file overlap" concerns, because those are inherently bipartite or matrix-friendly problems. A matrix can show intensity without edge clutter, while a bipartite layout can make issue-to-file structure easier to scan.

Inference from the sources: the best visualization upgrade is probably not "make the force graph prettier." It is "add a second representation optimized for dense comparison tasks."

## Question 6

### How should hotspot and progress visualization avoid becoming noise?

Visualization studies consistently punish decorative density. A view becomes valuable when it helps answer a question faster and with fewer errors. For Ishoo, that means hotspot and progress surfaces must stay tied to operational questions:

- which files attract too much concurrent work?
- what is blocked?
- where is progress concentrated?
- which sections are overloaded?

The graph literature and GitHub-style issue models both suggest that users benefit most when summaries are interpretable at a glance and drill down cleanly. That implies:

- ranking before ornament
- stable color semantics
- explicit counts
- easy transition from overview to underlying issues

For Ishoo's heatmap and timeline surfaces, that means bars, counts, and sortable lists are likely to outperform overly artistic visualization once density rises. The visualization should support judgment, not merely decorate the app.

Inference from the sources: Ishoo should prefer operationally legible views over visually ambitious ones, then reserve more expressive graphics for exploration once the simple questions are already answered well.

## Topic Synthesis for Ishoo

The interaction and visualization evidence points to three major conclusions.

First, drag quality needs to be treated as a specific motor-control and rerender-budget problem, not a generic animation problem. Slow drag is a distinct benchmark and should remain one of the app's hardest quality bars.

Second, accessibility should be designed around keyboard-operable reordering and disciplined focus management. A roving-focus grid or list model is more robust than trying to make drag semantics themselves carry the accessibility story.

Third, the current graph and hotspot surfaces should evolve toward multi-view representation. Node-link is valuable, but not sufficient for dense comparison work. Matrix-like and bipartite views are strong future candidates for overlap and dependency analysis.

The best near-term UI implications are:

- isolate drag-path reactivity aggressively in Dioxus
- design explicit keyboard reorder flows for feed and board
- keep feed and board parity grounded in shared movement logic
- add dense comparison views once graph clutter becomes a real product limit

## Sources

- W3C WAI-ARIA APG: [Grid Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/grid/)
- MDN: [aria-dropeffect](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-dropeffect)
- PubMed: [How should Fitts' Law be applied to human-computer interaction?](https://pubmed.ncbi.nlm.nih.gov/11539107/)
- PubMed: [Fitts' law model and target size of pointing devices in a vibration environment](https://pubmed.ncbi.nlm.nih.gov/18229550/)
- docs.rs: [dioxus_signals](https://docs.rs/dioxus-signals)
- Dioxus docs: [Signals and use_signal](https://dioxuslabs.com/learn/0.6/guide/state)
- docs.rs: [Signal](https://docs.rs/dioxus/latest/dioxus/prelude/dioxus_signals/struct.Signal.html)
- PubMed: [Node-Link or Adjacency Matrices: Old Question, New Insights](https://pubmed.ncbi.nlm.nih.gov/30130228/)
- PubMed: [Comparative Evaluation of Bipartite, Node-Link, and Matrix-Based Network Representations](https://pubmed.ncbi.nlm.nih.gov/36191101/)
- BMC / PubMed: [Dynamic graph exploration by interactively linked node-link diagrams and matrix visualizations](https://pubmed.ncbi.nlm.nih.gov/34491465/)
- Eurographics: [Path Visualization for Adjacency Matrices](https://diglib.eg.org/items/b0eab342-f6dd-48c8-9883-f336526f2837)
