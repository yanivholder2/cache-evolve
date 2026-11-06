# Variant v003 Insight

## Helped

- hit-rate (web_0 @ 16384): Conservative update removes the collision mass that warm probation victims used to get from sharing counters with hotter keys. A returning key at count 2 can now tie or beat those victims, so web_0's long-reuse-distance keys win the duel and get admitted more often. This is the only cell that clearly moved upward, and the whole small rise in the summary comes from it.
- hit-rate (summary): The summary rose slightly. The gain does not come from the cell the hypothesis targeted. It comes from web_0 @ 16384 alone, and the other three cells all moved slightly down.

## Hurt

- hit-rate (hm_0 @ 16384): The predicted mechanism does not exist here. A one-shot candidate never gets past the doorkeeper, so it never touches the counters, and its estimate stays at the doorkeeper bit whatever the update rule. Conservative update can only lower estimates, and here it lowered the victims' scores toward that floor. That makes `cand_est >= victim_est` ties (which admit) more common, so the one-shot filtering got weaker, not stronger.
- hit-rate (hm_0 @ 131072): This cell slipped in the same way, even though it ages on the 16 × period. The change is not size-gated, and its main effect is to deflate victims, not to fix collision inflation that builds up under the longer 32 × aging period. That means the rationale's link to the long aging periods does not hold.
- hit-rate (web_0 @ 131072): No meaningful change. The sketch has 4 × capacity columns per row, so collisions between warm keys are already rare at this size, and conservative update has little to fix.

## Takeaway

On this sketch, collisions do not inflate one-shot keys. The doorkeeper already holds them at the floor, and conservative update only moves warm victims' scores down toward them. Because ties admit, tighter victim estimates helped long-reuse web_0 at small size but made hm_0's scan filtering looser at both sizes. If hm_0 is still the target, the levers are on the victim side or in the tie rule (for example, requiring the candidate to strictly beat the victim when the victim's count is above 1), not on making the counters more precise.
