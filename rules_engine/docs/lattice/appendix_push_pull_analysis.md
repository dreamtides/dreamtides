# Appendix: Push vs Pull Context Analysis

## The Question

When should a system proactively include related information ("push") versus
waiting for explicit requests ("pull")? This analysis examines the tradeoffs
for AI agent interactions with document systems.

## Definitions

**Push Context**: System automatically includes related documents based on
structural relationships, labels, or heuristics. The AI receives information
it didn't explicitly request.

**Pull Context**: AI explicitly queries for each piece of information needed.
The system returns only what was requested.

## Arguments for Push Context

### 1. Reduced Round-Trip Latency

Each tool call has latency overhead (typically 100-500ms). If an AI needs
information from 5 related documents, push context delivers in 1 call what
pull requires 5+ calls to achieve.

**Math example:**
- Push: 1 call × 300ms = 300ms
- Pull: 1 call + 5 follow-ups × 300ms = 1800ms

For time-sensitive tasks, push can be 5-6x faster.

### 2. Preventing Unknown Unknowns

AIs can only request documents they know exist. Push context surfaces relevant
information the AI didn't know to ask for:

- A design constraint documented elsewhere
- A related bug that provides useful context
- A recent decision that changes the approach

This is especially valuable for:
- New contributors unfamiliar with the codebase
- Complex domains with non-obvious dependencies
- Documents that should always be consulted together

### 3. Consistency and Completeness

Push context ensures certain information is always considered together. For
example, an authentication implementation should always be viewed alongside
the security policy document. Push makes this coupling explicit and automatic.

### 4. Reduced Cognitive Load on AI

Deciding what to request requires the AI to:
1. Model what information might be relevant
2. Formulate queries to find it
3. Evaluate results and iterate

Push context removes this planning burden for well-structured relationships.

## Arguments for Pull Context

### 1. Token Efficiency

Context window space is finite and shared. Push context may include:
- Information irrelevant to the current task
- Documents the AI has already processed
- Content that exceeds what the AI can effectively use

**Math example:**
- Budget: 5000 chars pushed, task needs 1000 chars
- Waste: 4000 chars (80%) consumed for marginal value

### 2. AI Agency and Relevance Judgment

The AI is better positioned to judge relevance than static heuristics because:
- It knows the current task requirements
- It knows what it has already learned
- It can adapt to task-specific needs

Static push rules can't account for this dynamic context.

### 3. Predictability and Debuggability

With pull context:
- AI knows exactly what information it has
- Behavior is reproducible given the same queries
- Debugging is straightforward (examine query and response)

With push context:
- Behavior varies based on document graph state
- Changes to "related" documents affect unrelated queries
- Harder to understand why AI made certain decisions

### 4. Avoiding Information Overload

Research on human cognition shows attention degrades with irrelevant
information. Similar effects may apply to AI attention mechanisms:
- More content = more to attend to
- Irrelevant content may distract from relevant
- Quality of understanding may decrease with quantity

## When Push Context Works Well

### Scenario 1: Well-Defined Relationships

When documents have explicit, intentional relationships:
- Document A defines the interface, Document B implements it
- Issue X is blocked by Issue Y
- Design doc covers the same system as implementation doc

These relationships are inherent, not task-dependent. Push makes sense.

### Scenario 2: Mandatory Context

Some information should always be considered:
- Legal/compliance requirements
- Security constraints
- API contracts that can't be violated

Push ensures this context is never overlooked.

### Scenario 3: Task Briefings

At task start, comprehensive context helps orientation:
- Issue details with all blocking issues
- Related design documents
- Recent related changes

This is a bounded, intentional push for a specific purpose.

### Scenario 4: Small Document Sets

With few documents, the cost of over-inclusion is low:
- 3 related documents totaling 2000 chars: include all
- 30 related documents totaling 50000 chars: selective pull

## When Pull Context Works Better

### Scenario 1: Large, Complex Document Graphs

When many documents are "related" but most aren't relevant:
- Monorepo with hundreds of design docs
- Issue trackers with long history
- Documentation wikis with extensive cross-linking

### Scenario 2: Focused, Specific Tasks

When the task is narrow and well-defined:
- Fix a specific bug with known location
- Answer a specific question
- Verify a single fact

### Scenario 3: Continuation Sessions

When the AI has accumulated context over multiple turns:
- Already loaded relevant documents
- Built mental model of the problem
- Knows what it needs next

### Scenario 4: Resource-Constrained Environments

When token budgets are tight:
- Long conversations approaching limits
- Multiple tools competing for context
- Cost-sensitive applications

## Lattice's Hybrid Approach

Lattice provides both models with explicit user control:

### Default: Configurable Push

```
lat show <id>                    # Push with default budget
lat show <id> --context 0        # Pure pull
lat show <id> --context 10000    # More push
```

### AI Mode: Default Pull

```
lat show <id> --ai               # No automatic context
lat show <id> --ai --context 5000 # Opt-in to push
```

### Task-Specific Push

```
lat show <id> --brief            # Comprehensive task briefing
lat show <id> --intent=bug-fix   # Task-appropriate context
```

### Explicit Pull Commands

```
lat links-from <id>              # What does this link to?
lat links-to <id>                # What links here?
lat similar <id>                 # Semantically related
```

## Recommendations

### For AI Agent Developers

1. **Start with pull, add push selectively**: Begin with explicit queries,
   add automatic context only where clearly beneficial.

2. **Use briefings for task start**: Push context at session start, then
   pull during execution.

3. **Monitor context efficiency**: Track what pushed content is actually
   used vs ignored.

### For Document System Designers

1. **Make push optional and configurable**: Users and AIs should control
   how much automatic context they receive.

2. **Provide explicit relationship queries**: Enable pull via dedicated
   commands for link traversal.

3. **Support task-aware context**: Intent-based flags beat static heuristics.

4. **Default conservatively**: Better to under-push and allow explicit
   requests than over-push and waste context.

## Conclusion

Neither pure push nor pure pull is optimal. The best approach:

- **Push** for intentional relationships, mandatory context, and task briefings
- **Pull** for exploration, specific queries, and resource-constrained scenarios
- **Configurable defaults** that match the most common use case
- **Explicit overrides** for both directions

Lattice defaults to moderate push (5000 char budget) for human users who
benefit from automatic context, and no push (AI mode) for agent users who
can make their own relevance judgments.
