---
kind: decision
id: B002
title: Briefs live inside Ishoo rather than becoming a standalone app
status: ADOPTED
related:
  issues: [66, 75, 76, 92]
  briefs: [B001, B003]
---

# Decision
The new project-memory / philosophy / research layer will be implemented as Briefs inside Ishoo, not as a fourth standalone app.

# Context
The suite already has three clear first-class apps:
- Ishoo for work
- SEMMAP for structure
- Neti for enforcement

There is a real need for durable project memory, but adding another standalone app would increase conceptual surface area and user-facing cognitive load.

# Alternatives considered
- Create a fourth standalone app for project memory
- Store all of this as loose docs outside the product
- Fold it into Ishoo as a typed second artifact family

# Rationale
The user should feel like they are adopting one coherent way of working, not a bureaucracy of tools.

Project memory is valuable, but most users do not want to consciously adopt a philosophy-management app. They want the system to remember important things in the right shape.

Ishoo is already the human decision surface for priorities, scope, and redirection. That makes it the natural home for Briefs.

# Consequences
- Ishoo will own both Issues and Briefs
- the issue-first workflow must remain simple and uncluttered
- Briefs must be typed and constrained so Ishoo does not become mush
- SEMMAP and Neti may link to or consume Briefs, but do not own them
