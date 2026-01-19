<!-- section:contract required -->
### Tree Agent Contract

<contract>
Tree agent contract:

- Decide whether the selected node should be executed now (`decision=execute`) or decomposed (`decision=decompose`).
- If `decision=decompose`, you MUST provide 1+ child specs in `children` (title, goal, optional acceptance).
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
