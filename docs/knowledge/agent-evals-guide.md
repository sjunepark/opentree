# Demystifying Evals for AI Agents

Source: [Anthropic Engineering Blog](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents) (January 9, 2026)

## Why Evals Matter

"Good evaluations help teams ship AI agents more confidently" by preventing reactive debugging cycles. Agents' multi-turn capabilities—tool calling, state modification, and adaptive behavior—create unique evaluation challenges compared to single-turn LLM interactions.

Teams without evals get bogged down in reactive loops. Early-stage adoption clarifies success criteria, preventing engineer interpretation divergence. Later-stage adoption prevents quality degradation.

## Core Terminology

| Term | Definition |
|------|------------|
| **Task/Problem** | Single test with defined inputs and success criteria |
| **Trial** | Individual attempt at a task; multiple trials generate consistent results |
| **Grader** | Logic scoring agent performance; tasks support multiple graders with assertions |
| **Transcript/Trace** | Complete record including outputs, tool calls, reasoning, and interactions |
| **Outcome** | Final environmental state (not just agent declarations) |
| **Evaluation Harness** | End-to-end infrastructure managing tasks, tools, recording, grading, and aggregation |
| **Agent Harness/Scaffold** | System enabling model-as-agent functionality |
| **Evaluation Suite** | Task collections measuring specific capabilities |

## Diagram: Simple vs Complex Evaluation

```text
Simple Evaluation:
┌────────┐    ┌────────┐    ┌─────────────┐
│ Prompt │───▶│ Output │───▶│ Grader Check│
└────────┘    └────────┘    └─────────────┘

Complex Multi-Turn Evaluation (e.g., Coding Agent):
┌──────────────────────────────┐
│ Inputs: Tools, Environment,  │
│         Task Description     │
└──────────────┬───────────────┘
               ▼
        ┌─────────────┐
        │ Agent Loop  │◀──┐
        │ (multi-turn)│───┘
        └──────┬──────┘
               ▼
        ┌─────────────┐
        │ Output:     │
        │ Implementation
        └──────┬──────┘
               ▼
        ┌─────────────┐
        │ Graders:    │
        │ Unit Tests, │
        │ Analysis    │
        └─────────────┘
```

## Three Grader Types

### 1. Code-Based Graders

**Methods:** String matching, binary tests, static analysis, outcome verification, tool-call verification, transcript analysis

| Strengths | Weaknesses |
|-----------|------------|
| Speed, cost efficiency | Brittleness against valid variations |
| Objectivity, reproducibility | Limited nuance |
| Debugging ease | Unsuitable for subjective tasks |
| Condition verification | |

### 2. Model-Based Graders

**Methods:** Rubric scoring, natural language assertions, pairwise comparison, reference-based evaluation, multi-judge consensus

| Strengths | Weaknesses |
|-----------|------------|
| Flexibility, scalability | Non-determinism |
| Nuance capture | Higher cost |
| Open-ended task handling | Calibration requirements |

### 3. Human Graders

**Methods:** SME review, crowdsourcing, sampling, A/B testing, inter-annotator agreement

| Strengths | Weaknesses |
|-----------|------------|
| Quality gold standard | Expense, slowness |
| Expert alignment | Expert access limitations |
| Calibration reference | |

## Evaluation Categories

**Capability/Quality Evals:** Assess agent potential; typically start low pass-rate and measure improvement trajectory

**Regression Evals:** Maintain ~100% baseline; detect performance degradation and ensure changes don't introduce unintended consequences

## Agent-Type-Specific Approaches

### Coding Agents

Leverage deterministic pass/fail testing from unit tests. Benchmarks: SWE-bench Verified, Terminal-Bench.

**Example Structure:**

```text
Task: Fix authentication bypass vulnerability
Graders:
  1. Unit tests pass
  2. Static analysis clean
  3. Security log verification
  4. Specific tool-call sequences verified
```

### Conversational Agents

Multi-dimensional success metrics: task completion, turn limits, interaction tone. Often use second LLM simulating user personas.

**Example Structure:**

```text
Task: Support refund request
Graders:
  1. LLM rubrics (empathy, clarity, grounding)
  2. State verification (resolved tickets, processed refunds)
  3. Required tool calls present
  4. Turn constraints met
```

### Research Agents

Unique challenges: Expert disagreement on comprehensiveness; constantly shifting ground truth; open-ended output length.

**Grader Dimensions:**

- Groundedness (claim support)
- Coverage (key facts)
- Source authority
- Coherence

### Computer Use Agents

Navigate GUIs via screenshots and input. Evaluate in sandboxed environments checking outcome achievement.

**Trade-off:** Token efficiency (DOM extraction) vs latency (screenshots)

## Non-Determinism Metrics

```text
               │
        100% ──┤         ╭─────── pass@k (≥1 success in k trials)
               │       ╭─╯
               │     ╭─╯
  Pass Rate    │   ╭─╯
               │ ╭─╯
        50% ──┤╭╯
               │╲
               │ ╲
               │  ╲───────────── pass^k (all k trials succeed)
               │    ╲
         0% ──┤      ╲─────────
               └─────┬─────┬─────┬──▶
                    1     5    10
                         k trials
```

**pass@k:** Probability of ≥1 correct solution across k attempts. Increases with trials.

**pass^k:** Probability all k trials succeed. Decreases with trials. Critical for consistency-demanding customer-facing agents.

At k=1, both equal per-trial success rate. By k=10, pass@k approaches 100% while pass^k may near 0%.

## Implementation Roadmap (8 Steps)

### Steps 0-2: Initial Dataset Collection

1. Begin with 20-50 simple tasks from real failures
2. Convert existing manual checks and user-reported bugs into test cases
3. Ensure task specifications pass expert review; create reference solutions proving solvability

### Step 3: Balanced Problem Sets

Test both "should occur" and "shouldn't occur" scenarios. Avoid class imbalance.

**Example:** Claude.ai search balancing undertriggering vs. overtriggering

### Steps 4-5: Harness & Grader Design

- Isolate trials with clean environments preventing state leakage
- Grade outputs/outcomes over specific step sequences (agent creativity acceptable)
- Implement partial credit for multi-component tasks

### Step 6: Transcript Review

> "You won't know if your graders are working well unless you read the transcripts and grades from many trials."

This catches grading bugs and validates fairness.

### Step 7: Saturation Monitoring

Track when evals approach 100% pass-rates, indicating diminishing improvement signals.

**Example:** SWE-Bench Verified progressed 30% → >80% within one year

### Step 8: Long-Term Maintenance

- Establish dedicated ownership
- Enable domain experts and product teams as contributors
- Practice eval-driven development: define evals before capability implementation

## Complementary Evaluation Methods

| Method | Strengths | Weaknesses |
|--------|-----------|------------|
| **Automated Evals** | Fast iteration, reproducibility, CI/CD integration | Upfront investment, maintenance drift, false confidence |
| **Production Monitoring** | Real user behavior, ground truth, scales | Reactive, post-deployment, noisy signals |
| **A/B Testing** | Actual user outcomes, confound control | Weeks-long timelines, only deployed variants |
| **User Feedback** | Unexpected problem discovery, real examples | Sparse, self-selected, severe-issue bias |
| **Manual Transcript Review** | Intuition building, subtle issue detection | Time-intensive, inconsistent coverage |
| **Systematic Human Studies** | Gold-standard judgments, subjective domains | Expensive, slow, inter-rater reconciliation |

**Post-launch strategy:**

1. Automated evals → first-line defense
2. Production monitoring → drift detection
3. A/B testing → traffic-sufficient changes
4. Ongoing user feedback and transcript sampling

## Real-World Examples

| Product | Approach |
|---------|----------|
| **Claude Code** | Evolved: employee feedback → narrow evals (concision, edits) → complex behaviors (over-engineering) |
| **Descript** | Video editing evals spanning three success dimensions; manual → LLM grading with human calibration |
| **Bolt** | Added evals three months post-launch using static analysis, browser agents, and LLM judges |
| **Opus 4.5 / CORE-Bench** | Initial 42% → 95% after fixing rigid grading (exact matching vs. acceptable ranges), ambiguous specs, and stochastic tasks |

## Open-Source Frameworks

| Framework | Description |
|-----------|-------------|
| **Harbor** | Containerized environment infrastructure; benchmarks via registry |
| **Promptfoo** | Lightweight YAML-based configuration; assertion types from string-matching to LLM rubrics |
| **Braintrust** | Offline evaluation + production observability + experiment tracking |
| **LangSmith** | LangChain-integrated tracing and evaluation |
| **Langfuse** | Self-hosted open-source alternative |

## Key Takeaways

1. **Treat evaluations as core infrastructure**, not afterthought
2. **Combine multiple approaches:** automation for speed, monitoring for real-world drift
3. **Read transcripts:** Manual review catches grading bugs and builds intuition
4. **Balance problem sets:** Test both positive and negative cases
5. **Monitor saturation:** Refresh evals when they approach 100%
6. **Practice eval-driven development:** Define evals before implementing capabilities
7. **Grade outcomes, not steps:** Allow agent creativity in reaching goals
