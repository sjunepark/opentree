<!-- section:contract required -->
### Executor Contract

<contract>
Executor contract:

- Do not modify passed nodes.
- Do not set `passes=true` (runner-owned).
- You MAY edit open nodes in `.runner/state/tree.json` (title, goal, acceptance), but:
  - MUST NOT add children to any node
  - MUST NOT change `next`, `passes`, or `attempts` (runner-owned fields)
- Run formatting/lint/tests as appropriate before declaring `status=done`.
- Final response must be a single JSON object matching the output schema (no markdown, no code fences).

</contract>

{% if planner_notes %}
<!-- section:planner droppable -->

### Planner Notes

<planner>{{ planner_notes }}</planner>

{% endif %}
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
