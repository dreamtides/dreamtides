# Appendix: AI Friendliness Analysis

## Overview

This appendix examines Lattice's AI-friendliness design decisions with a critical
eye toward what works, what doesn't, and how the system could better serve AI
agents operating under real-world constraints.

## Current AI-Friendly Features

Lattice incorporates several features explicitly designed for AI consumption:

### Document Size Limits

The 500-line soft limit prevents monolithic documents that consume excessive
context window space. This is a sound principle: atomicity enables selective
loading and prevents the "all or nothing" problem where a 10,000-line document
must be loaded in full or not at all.

**Assessment**: Good foundation. The limit is appropriately soft (warning, not
error) since some documents legitimately need more space.

### Structured Metadata

YAML frontmatter provides machine-readable metadata separate from prose content.
The `--peek` command loads only frontmatter, enabling cheap document scanning.

**Assessment**: Effective. Separating metadata from content lets AI agents
quickly triage documents without loading full bodies.

### Stable Identifiers

Lattice IDs provide stable, human-typeable references that survive document
moves and renames. This is superior to path-based references for AI workflows
where document locations may change during a task.

**Assessment**: Strong design choice. Stable IDs reduce brittleness in
multi-step AI workflows.

### AI Mode Output

The `--ai` flag produces output optimized for AI parsing with bracketed markers
and no decorative elements. Defaults to no automatic context.

**Assessment**: Useful but potentially underspecified. See recommendations below.

## Push vs Pull Context: Analysis

The fundamental tension in context delivery is between proactive inclusion
(push) and explicit retrieval (pull). Both have legitimate use cases.

### When Push Context is Effective

**1. Task Initialization**

At the start of a task, the AI has minimal knowledge of what it needs. Push
context via `--brief` provides comprehensive orientation without requiring the
AI to know what to ask for.

This works because:
- Unknown unknowns are highest at task start
- The cost of over-inclusion is amortized over the entire task
- Briefings are bounded events, not continuous overhead

**2. Mandatory Coupling**

Some documents should never be read in isolation:
- Security policies alongside authentication implementations
- API contracts alongside their implementations
- Error code definitions alongside error handling docs

Push makes these couplings explicit and automatic. The alternative (relying on
AI agents to always remember to fetch security policies) is fragile.

**3. Small, Well-Structured Repositories**

In repositories with <100 documents and clear organizational structure, the
cost of over-inclusion is low. A 5000-character budget can meaningfully cover
related content without waste.

### When Pull Context is More Effective

**1. Focused Operations**

For targeted tasks ("find where error X is raised"), push context is overhead.
The AI knows exactly what it needs and should retrieve it directly.

**2. Continuation Contexts**

After multiple turns in a conversation, the AI has accumulated significant
context. Push behavior that was helpful at turn 1 becomes wasteful at turn 10
when the AI already knows the relevant documents.

**3. Large, Complex Document Graphs**

In repositories with thousands of documents and dense cross-linking, push
heuristics break down. "Related" becomes too broad a category when everything
transitively relates to everything else.

**4. Resource-Constrained Sessions**

When context window space is tight (long conversations, multiple tools), every
character matters. Pull gives the AI control over this scarce resource.

### The Hybrid Problem

Lattice's current hybrid approach (configurable push via budgets) has a subtle
problem: it places the burden of choosing push vs pull on the AI or user at
invocation time, but the optimal choice depends on information not available
at that moment.

Consider: Should `lat show LXXXX` include 5000 characters of context?
- If this is the first document in the task: probably yes
- If the AI already loaded 3 related documents: probably no
- If the AI has 50K tokens remaining: yes
- If the AI has 5K tokens remaining: no

The system can't know these factors without deeper integration.

## Critique of Current Design

### Problem 1: AI Mode is Too Passive

The `--ai` flag defaults to no automatic context, which is conservative but
potentially too conservative. An AI loading a document likely needs *some*
context, or it wouldn't be loading the document.

The current design puts the burden on the AI to remember to request context
when needed, which is exactly the failure mode push context exists to prevent.

### Problem 2: Context Budgets are Character-Based

Character budgets assume all characters have equal value, but this is false:
- YAML frontmatter is denser (more information per character)
- Prose explanations are sparser
- Code examples are variable

A 5000-character budget might include 20 short metadata entries or 1 long
prose document. The former is almost certainly more valuable for orientation.

### Problem 3: No Session Awareness in Context Selection

The context algorithm operates statelessly: each `lat show` runs the same
algorithm regardless of what the AI has already seen. This leads to:
- Repeated inclusion of already-loaded documents
- No learning from what the AI actually uses
- No optimization across a session

### Problem 4: Intent Model is Limited

The `--intent` flag is a step toward task-aware context, but the current
intents (implement, bug-fix, review, understand, document) are coarse. Real
tasks exist on a spectrum:
- "Implement feature X" is different from "implement feature Y"
- "Fix bug A" may need different context than "fix bug B"

Static intent categories can't capture this variation.

### Problem 5: Incremental Loading is Manual

The `--peek`, `--sections`, and `--section` flags enable incremental loading,
but using them requires the AI to execute a multi-step workflow:
1. Load frontmatter
2. Decide if body needed
3. Load sections list
4. Decide which sections
5. Load specific sections

This workflow taxes AI planning capabilities and burns tool call budget on
meta-operations rather than actual work.

## Recommendations for Improvement

### Recommendation 1: Token-Based Budgets

Replace character budgets with approximate token counts. Most AI systems have
predictable tokenization, and token budgets more directly reflect the actual
constraint.

```
lat show <id> --context 1000t    # 1000 tokens
lat show <id> --context 5000c    # 5000 characters (backward compat)
```

### Recommendation 2: Progressive Disclosure by Default

Instead of all-or-nothing document loading, default to a progressive format:

```
[DOCUMENT:LXXXX:auth-design]
[SUMMARY]
This document describes the authentication architecture.
[/SUMMARY]
[SECTIONS]
- Introduction
- OAuth Flow (420 tokens)
- Session Management (380 tokens)
- Error Handling (290 tokens)
[/SECTIONS]
[EXPAND:section_name] to load specific content
[/DOCUMENT]
```

The AI receives orientation information (summary, section list with sizes)
without full content, then explicitly requests sections as needed.

This shifts from "push vs pull" to "push structure, pull content."

### Recommendation 3: Session-Aware Context

Track what the AI has loaded within a session and exclude already-seen content
from context:

```
lat show <id> --session <session-id>
```

The system maintains session state and:
- Excludes documents already shown this session
- Tracks AI requests to learn what content is actually useful
- Adjusts context selection based on session history

### Recommendation 4: Structured AI Output Format

Define a richer AI output format with explicit semantic regions:

```
[DOCUMENT:LXXXX:auth-design]
[META]
type: knowledge-base
labels: authentication, security
priority: 2
related: LYYYY, LZZZZ
[/META]
[BODY]
Document content here...
[/BODY]
[ACTIONS]
- [SHOW:LYYYY] View OAuth implementation
- [SHOW:LZZZZ] View session handling
- [SEARCH:authentication errors] Find related content
[/ACTIONS]
[/DOCUMENT]
```

This format:
- Clearly separates metadata, content, and navigation options
- Makes available actions explicit
- Provides structured handles for AI tool calls

### Recommendation 5: Adaptive Context Based on Remaining Budget

Accept optional context about the AI's current state:

```
lat show <id> --ai --remaining-tokens 8000 --session-turn 5
```

With this information, the system can:
- Adjust context budget based on available space
- Be more aggressive at turn 1, more conservative at turn 10
- Prioritize differently when resources are tight

### Recommendation 6: Content Importance Scoring

Allow documents and sections to declare relative importance:

```yaml
---
lattice-id: LXXXX
doc-importance:
  summary: critical    # Always include
  "OAuth Flow": high   # Include if budget allows
  "History": low       # Only if explicitly requested
---
```

This enables smarter truncation: important sections survive tight budgets,
low-importance sections are elided first.

### Recommendation 7: Tool-Integrated Mode

For AI systems with tool calling, provide operations as structured tool schemas
rather than CLI commands:

```json
{
  "lattice_operations": [
    {"name": "show", "args": {"id": "LXXXX"}, "description": "View auth design"},
    {"name": "search", "args": {"query": "timeout"}, "description": "Find timeout docs"},
    {"name": "expand", "args": {"id": "LXXXX", "section": "OAuth Flow"}}
  ]
}
```

This integrates naturally with AI tool use patterns rather than requiring
string-based command construction.

## The Fundamental Tradeoff

The core tension between push and pull reflects a deeper question: who should
bear the cognitive load of relevance judgment?

**Push assumes**: The system knows better than the AI what's relevant.
**Pull assumes**: The AI knows better than the system what it needs.

Both assumptions are partially true:
- The system knows structural relationships the AI can't discover cheaply
- The AI knows its current task requirements and context state

The ideal design acknowledges both: the system provides structure and hints,
the AI makes final decisions about what to load. This suggests:

1. Always push metadata and structure (cheap, enables AI decisions)
2. Let AI pull content (expensive, task-dependent)
3. Support session state for cross-turn optimization
4. Make context reasoning transparent (show what's available, let AI choose)

## Conclusion

Lattice's current AI-friendliness features are a reasonable starting point but
reflect first-generation thinking about AI-document interaction. The system
treats AI agents as slightly-different humans rather than as fundamentally
different consumers with distinct needs.

The recommendations above move toward a model where:
- Structure is always pushed (orientation is cheap and always valuable)
- Content is pulled on demand (body text is expensive and variably valuable)
- Session state enables cross-turn learning (no wasted re-loading)
- Output format is machine-optimized (not human-readable with AI flag)

This represents a shift from "documents with AI mode" to "AI-native document
retrieval," recognizing that AI agent needs are different enough to warrant
distinct interfaces rather than parameterized versions of human interfaces.
