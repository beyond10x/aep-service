---
format: aep.planning-md/1
id: decision-blocker:cross-store-vision-targets
kind: decision-blocker
status: open
title: 'The 56 serves: edges cannot name atlas as their target'
summary: No spelling names an artifact in another workspace member, so the four archived vision mirrors that hold edges stay as local targets.
relations:
- blocks: vision:O1
- blocks: vision:O2
- blocks: vision:O4
- blocks: vision:O6
revision: 1
---
## What is blocked

Fifty-six `serves:` edges in this store point at `vision:O1`, `vision:O2`, `vision:O4` and
`vision:O6` — 14, 30, 1 and 11 of them respectively. All four targets are `archived`: decision item
**18** of the 2026-09-15 org-state review named atlas the authority for the objectives and retired
the six local mirrors (`ed4ef5b`). The edges were left pointing at the retired mirrors, and
`aep plan artifact validate --strict` is clean on that, because every target does still exist in
this store. Nothing is broken; something is misleading. A reader who follows
`story:authorized-reads serves vision:O1` lands on a document that was a hand-kept snapshot of
somebody else's row, and the store gives them no signal that the authority is elsewhere.

`vision:O3` and `vision:O5` hold no edge at all — no artifact in this store declares `serves:
vision:O3` or `serves: vision:O5`. They are archived mirrors that nothing points at.

## Why the edges were not repointed

There is no spelling that names an artifact in another workspace member as a relation target.
The workspace is declared and the reference resolves — `.engineering/workspace.yaml` names `atlas`
at `../../atlas`, and from a checkout where that member is present:

    $ aep plan workspace show atlas/vision:O1
    atlas/vision:O1  draft  Governed reach            # exit 0

    $ aep plan workspace show vision:O1
    vision:O1 is held by 2 members; say which:
      aep-service/vision:O1
      atlas/vision:O1                                 # exit 1

— but `member/kind:name` is readable only by the `workspace` verbs. The two writing verbs refuse it,
and they refuse it at two different depths:

    $ aep plan artifact relate story:authorized-reads serves atlas/vision:O1
    error: … does not hold `atlas/vision:O1`, so `story:authorized-reads serves
    atlas/vision:O1` would be an edge to nothing                     # exit 1

    $ aep plan artifact new story probe --relate serves:atlas/vision:O1
    error: `atlas/vision:O1` cannot be given an address: invalid locator identifier
    "ep://planning/store/atlas/vision/O1": the kind contains disallowed
    character '/'                                                    # exit 1

The first is a resolution refusal — the verb looked in one store. The second is a grammar refusal:
an artifact id has a `kind` and a `name` and nowhere to put a member, so `atlas/vision` is read as a
kind and rejected for the slash. That is the real obstacle; the first refusal would still stand
after the second was fixed.

Hand-writing the frontmatter is not an alternative: `- serves: atlas/vision:O1` written into a file
directly makes `validate` refuse the artifact as drift — *"drifted from its log: relations disagrees
with event story:authorized-reads@1#0~5dbec66fd21b286d — an edit made outside a command is a change
nothing decided"* (exit 1, `--strict` likewise). So the store is closed to the edge in both
directions: no command will write it, and no hand-edit survives validation.

Decision item **7** of the same review left the cross-store relation form unadopted, so this is a
known gap being recorded, not a discovery.

## What the missing verb would need to be

Four things, in the order they have to be built:

1. **A member slot in the reference the write verbs accept.** `aep plan artifact relate <id>
   <relation> <member>/<kind>:<name>` and `aep plan artifact new --relate
   <relation>:<member>/<kind>:<name>` must parse `<member>/` and keep it, rather than folding it
   into the kind. `aep plan workspace show` already parses exactly that spelling, so the grammar
   exists; it is the id type underneath that has no field for it. Either the locator gains a member
   segment — `ep://planning/store/<member>/<kind>/<name>` rather than today's
   `ep://planning/store/<kind>/<name>` — or the member travels beside the id and never inside it.
   The second is the smaller change and keeps every existing id byte-identical.

2. **Resolution through the workspace file, not the store.** `relate` currently asks *this* store
   whether it holds the target. A member-qualified target must be resolved through
   `.engineering/workspace.yaml`, against the member's own store, and must be refused when the
   member is not declared — an undeclared member is a typo, and that refusal is the one worth
   keeping.

3. **An absent member must not be a validation failure.** `aep plan workspace members` already
   reports a member nobody has checked out as `absent` and the workspace file says in its own words
   that this is a normal condition, not a broken workspace. So `aep plan artifact validate` must
   treat a crossing edge whose member is absent as resolvable-in-principle and stay silent, and the
   unresolved case must surface at `aep plan workspace crossings --strict`, which exists for exactly
   that and today reports `0 crossing relation(s), 0 unresolved, 0 cycle(s)` only because no
   crossing edge can be authored. Putting it in `validate` instead would turn every checkout without
   a sibling atlas red, including CI.

4. **`supersedes` across the boundary, or an explicit decision not to.** The retirement these six
   mirrors record cannot be written down as an edge either: `supersedes` runs from the replacement to
   the replaced, the replacement is `atlas/vision:O1`, and it lives in the store that would have to
   declare it. Until 1–3 exist, the supersession is prose in six bodies. Whether atlas should
   declare it, or whether this store should be able to declare the reverse, is the part of item 7
   that was deferred rather than answered.

## What clears this

Either an AEP release whose `aep plan artifact relate` accepts a member-qualified target — at which
point the 56 edges are repointed at `atlas/vision:O1`, `O2`, `O4` and `O6`, all six mirrors are left
with nothing pointing at them, and this blocker is `cleared` — or a decision that cross-store
relations will not be adopted, in which case the six mirrors are permanent and this record says why.
Both are decisions; neither is work this repository can do alone, which is why it is a
`decision-blocker` rather than a story.
