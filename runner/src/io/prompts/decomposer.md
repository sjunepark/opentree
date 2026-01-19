<!-- section:contract required -->
### Decomposer Contract

<contract>
Decomposer contract:

- You must decompose the selected node into child specs.
- Provide 1+ child specs in `children` with `title`, `goal`, `acceptance` (optional), and `next`.
- Output `children` in the order they should be worked on (task order). The runner assigns `order` from the array index.
- `next` controls what happens when that child becomes the selected leaf (`execute` or `decompose`).
- Do NOT edit repository files in this step.
- The runner owns `.runner/state/tree.json`; do NOT try to edit it to add children.

</contract>

<!-- section:sizing required -->
### Task Sizing

- Each child should be completable in ONE agent session (~30 min, full tool access)
- **Writing is execution**: specs, docs, code, tests are all `next=execute`
- Only use `next=decompose` for genuinely multi-phase work

### When to use each `next` value

**`next=execute`** (default for most children):
- Goal involves writing/editing files
- Can be verified with 3-5 acceptance criteria
- A capable agent could complete it end-to-end

**`next=decompose`** (rare):
- Goal spans multiple independent concerns (e.g., "auth system" = schema + middleware + UI)
- Cannot define clear acceptance without breakdown

### Anti-patterns (AVOID)

- Decomposing "write specs" or "define behavior" → writing IS executable work
- Splitting "implement" / "test" / "format" → these are one workflow
- Creating one child per edge case → group related work
- Deep trees (3+ levels) for simple goals

### Example

Goal: "Build Go CLI calculator with +,-,*,/"

BAD (over-decomposed):
```
├── Define CLI behavior (decompose) → leads to 6+ spec children
├── Implement CLI
├── Tests
└── Docs
```

GOOD:
```
├── Implement calculator with operations, error handling, tests (execute)
└── Update README with usage examples (execute)
```

<!-- section:goal required -->
### Goal

<goal>{{ goal }}</goal>

{% if history %}
<!-- section:history droppable -->

### History (previous attempt)

<history>{{ history }}</history>

{% endif %}
{% if failure %}
<!-- section:failure droppable -->

### Failure (guard output)

<failure>{{ failure }}</failure>

{% endif %}
<!-- section:selected required -->

### Selected Node

<selected>
path: {{ selected.path }}
id: {{ selected.id }}
title: {{ selected.title }}
goal: {{ selected.goal }}
next: {{ selected.next }}
{% if selected.acceptance %}acceptance:
{% for item in selected.acceptance %}- {{ item }}
{% endfor %}{% endif %}</selected>

{% if tree_summary %}
<!-- section:tree droppable -->

### Tree Summary

<tree>{{ tree_summary }}</tree>

{% endif %}
{% if assumptions %}
<!-- section:assumptions droppable -->

### Assumptions

<assumptions>{{ assumptions }}</assumptions>

{% endif %}
{% if questions %}
<!-- section:questions droppable -->

### Open Questions

<questions>{{ questions }}</questions>

{% endif %}
