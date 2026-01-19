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
