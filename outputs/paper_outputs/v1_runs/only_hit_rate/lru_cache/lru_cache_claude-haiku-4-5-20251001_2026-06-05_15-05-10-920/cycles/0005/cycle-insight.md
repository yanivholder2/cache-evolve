# Cycle 0005 Insight

## Cohort outcome

This cycle tested refinements to the Cycle 0004 winner (0.75× multiplicative frequency decay at `capacity * 16` intervals) by exploring coefficient sensitivity: increasing decay gentleness to 0.8× (v001), accelerating decay to 0.7× (v002), and replacing multiplicative decay with exponential approximation (v003). All three variants failed to improve upon the parent strategy; v002 and v003 actively regressed, while v001 showed no measurable benefit. The cache has reached a plateau around the current decay tuning.

## Patterns

- **The 0.75× decay coefficient is robustly tuned at a local optimum.** Adjustments in both directions—gentler (0.8×, approaching 1.0×) and more aggressive (0.7×, approaching 0.5×)—fail to yield improvements. V001's neutrality and V002's regression bracket the current value, indicating the workload's decay sensitivity converges at this specific coefficient, not as part of a broader continuum.

- **Exponential decay does not improve on multiplicative decay.** Although theoretically motivated by non-linear forgetting curves, the integer approximation (0.74 ≈ e^(-0.32)) introduces slightly more aggressiveness than 0.75× and undermines the frequency discrimination that the parent variant carefully preserved. The simpler linear multiplicative approach has already captured the optimal balance between forgetting and discrimination.

- **The cache is converging to a tight local optimum.** The failure of all three orthogonal perturbations suggests the current strategy has captured the major structural advantages available. Small changes to decay mechanics now hurt rather than help, implying the bottleneck for further improvement lies outside the decay mechanism itself.

## Open questions

- Is there a micro-tuned decay coefficient between 0.75× and 0.8× (e.g., 0.77×, 0.76×) that could extract marginal gains, or is the plateau independent of fine-grained coefficient adjustment?
- What aspect of the access pattern makes decay coefficient so critical? Is the workload's frequency distribution multimodal, causing sensitivity to forgetting rates, or are phase shifts the dominant factor?
- Could orthogonal decay mechanics—such as threshold-dependent decay (only decaying frequencies above a certain level), piecewise decay (different rates for hot vs. cold items), or frequency-dependent decay coefficients—circumvent the plateau without changing the baseline coefficient?

## Continuation

The 0.75× multiplicative decay approach from Cycle 0004 remains the strongest baseline and should be locked in as the foundation for Cycle 0006. **Abandon coefficient sensitivity testing**: both higher (0.8×) and lower (0.7×) values regress or stagnate, confirming the plateau is fundamental, not a missed optimization. **Do not pursue exponential or non-linear decay patterns** that approximate or replace multiplicative decay; the added complexity does not compensate for loss of frequency discrimination. For Cycle 0006, shift exploration away from decay mechanics entirely and probe orthogonal levers: test the `capacity * 16` reset interval for further optimization, examine whether frequency quantization or granularity improvements could sharpen discrimination, or investigate tie-breaking rules within equal-frequency items to improve hit-rate without modifying the decay mechanism itself.
