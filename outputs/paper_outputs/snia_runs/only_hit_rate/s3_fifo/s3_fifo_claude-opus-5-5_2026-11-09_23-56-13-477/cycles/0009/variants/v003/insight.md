# Variant v003 Insight

## Helped

- hit-rate (hm_0 @ 16384): The correlated-reference cap is working here, on the trace the hypothesis did not target. On hm_0, keys whose only small-queue hits come within `small_size / 8` insertions of admission used to reach freq 2 and graduate into a full main, where each one forced an `evict_main`. Now they leave small with freq 1 and become ghosts. The existing freq ≥ 1 ghost→main path still admits them on a real later return, so main stops spending slots on burst-only keys, and keys that really are reused lose only one miss.
- hit-rate (hm_0 @ 131072): The same effect, but smaller. The window scales with `small_size`, so the rule does reach the large size, as the rationale predicted. Fewer full-main graduations are decided at this capacity, though, so less main churn is avoided.

## Hurt

- hit-rate (web_0 @ 16384): On web_0, many keys that are truly reused get their second hit soon after insertion. The cap counts that hit as part of the insertion burst, so those keys now miss graduation and go to the ghost. Each one pays an extra miss on the ghost round trip before it reaches main. This is the opposite of the pollution filtering the hypothesis expected on web_0.

## Takeaway

Collapsing bursts of small-queue hits right after insertion into one reuse removes main pollution on hm_0. It does not help web_0: there, near-insertion reuse is real evidence, not correlated noise, and web_0 @ 131072 stays flat. The web_0 pollution found in cycle 0001 is therefore not caused by keys that are only reused in bursts, so this window will not lift the web_0 plateau. The hm_0 gain comes cheaply because a capped key keeps freq 1 into the ghost, which turns the cap into "defer main admission until a spaced return" rather than rejecting the key. If this mechanism is kept, the next knob is `CRP_DIVISOR`. A narrower window, meaning a larger divisor, might keep hm_0's gain while sparing web_0's quick reuses.
