# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Deepening the ghost window to `ghost_size = capacity` (≈2× the old `main_size` bound) via the new field and the two `>= self.ghost_size` trim comparisons left hit-rate a hair below the parent — the longer history admitted no extra useful re-admits and only retained staler keys.

## Takeaway

On this LIRS trace, ghost *depth* stays inert even after cycle-0006 made the ghost queue hold strong-reuse, main-evicted keys: a key that cooled to `freq == 0` and is reaccessed does so within the original `main_size` window or not at all, so the extra entries past `main_size` never convert to re-admissions. The hypothesis that ghosting the stronger population would wake the previously-dead depth lever is falsified — the win from cycle 0006 came from the ghost *source*, not the window length.
