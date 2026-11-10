# Build plan: one lane, in order, with acceptance numbers

*Design stage, 2026-11-10, base revision 6a0602d. The design is [design.md](design.md); the
measured problems are [problems.md](problems.md); the consonant model and its acceptance
lines (K1–K4, S1–S9, F1–F6, H1, A1, N1, L1, V1) are [consonants.md](consonants.md) §9.*

## How to work through it

- **Order.** What the operator hears first comes first: the consonants (step 1), on today's
  tubes. Then the reference solver (2), the real-time voice from the 3D geometry (3), the
  consonants' place and spectra from that geometry (4), the precise render (5), speed (6),
  the app's action (7, optional), the judges and the comparison (8).
- **Every step that changes sound**: small commits; renders and spectrograms of the
  operator's phrase and of the touched consonants looked at before moving on; the battery
  table; the speech judge; the singer comparison; the bench. Goldens are re-blessed once per
  step, at its end (`CHIMERA_BLESS=1 cargo test --workspace --no-fail-fast`, twice), then
  `bun run test` and `bun run typecheck`.
- **Evidence** outside git: `$LLV_EVIDENCE_DIR/acoustics3d/<step>/` (WAVs, spectrograms,
  traces, tables, logs). Numbers alone go in `docs/acoustics3d/results.md`, written as the
  steps finish; nothing derived from the MRI data but this project's error figures.
- **Scratch.** Builds, renders and probes run in an export of the checkout, never in the live
  worktree; no `target/` is shared between checkouts.
- **Other open work.** `anatomy/tract3d/`, `src/ui/`, `docs/tract3d/` have another owner:
  additive interfaces only (design.md §5.7). Before finishing, compare the base with `main`
  again and re-run what moved.
- **If the work grows past its estimate**, stop and say so (AGENTS.md). The cut order is at
  the end.

## Step 0. Baseline and measuring tools (no sound change)

1. Toolchain, `bun install`; `bun run test` and `bun run typecheck` on the base, logs kept.
2. Bring this stage's tools into the repository, as new files beside what exists:
   - `chimera probe text|ipa <stem> …`: the trace writer (the evidence folder's
     `scripts/probe`), and `tools/consonants/` with `battery.sh`, `analyze.py`, `tables.py`,
     `can.py`, `harmonicity.py` (they run in the speech judge's Python environment);
   - `tools/asr/wordtest.py` and `tools/asr/transcribe.py`. `speech_wer.py`,
     `sentences.tsv` and `setup.sh` stay as they are.
3. The singer's data (`tools/fetch-csd.sh`) and its baseline (`chimera compare csd`), the
   speech judge's baseline, the benches (native example, `tools/bench-wasm.ts` with a
   warm-up pass), all on the base commit.
4. One natural voice per sex for line V1: a 48 kHz recording under a free licence, kept
   outside git, measured by `harmonicity.py`.

**Acceptance.** On the base commit the tools reproduce problems.md: the tables of §4 to the
printed digit, word error rate 35.76 % training and 42.0 % held-out, word test 28 of 240,
native 18–21 % and wasm 31–33 % of a core on the design machine (elsewhere: the ratio wasm
to native within 1.5–1.8). The singer's score and guards are recorded as the base for
every later comparison.

## Step 1. Consonants: the air supply, the sources, the timing

On today's tubes; each item is its own commit, with the cause's reproduction kept as a test
(`tests/consonant_artifacts.rs` style: new tests, the existing file untouched).

| Item | Change (consonants.md) | Lines it must bring in |
|---|---|---|
| 1a | lung termination: the airway's resistance for steady flow, absorbing for sound (§1) | F6: voiceless fricatives hold 4.5–7.5 cmH2O; subglottal pressure during aspiration ≥ 6 cmH2O at lung pressure 8 |
| 1b | one turbulence law for the glottal jet, free jets and jets on obstacles; bands that follow the jet; the source on while the jet is turbulent; the source present at the junction of the tubes (§3, §4) | S3, S4 (with 1c), S7, F1, F2, F5, H1, V1 |
| 1c | opening speed by articulator (§2) | S8, S4 |
| 1d | voice onset by place, stress and what follows (§2) | S5, S6, K2, K3 |
| 1e | voiced stops and fricatives hold pressure (§2, §3) | S2, F4 |
| 1f | the thump before a phrase-initial closure: isolate which stage gives the displaced air its speed, then fix that stage (problems.md §5.9) | K4, S9 |
| 1g | affricates seal and keep their frication (§5) | S1, A1 |
| 1h | the female source by fold length, output re-levelled (§3) | the "female within 3–4 dB of male" parts of S3, F1 |

**Acceptance at the end of the step.**
- K1–K4 on the operator's phrase; S1–S9, F1, F2, F4–F6, H1, A1 on the battery and the
  training sentences, both voices. (F3, the sibilants' peaks, may wait for step 4 if the
  1D front cavity cannot place them; say which.)
- Speech judge: training word error rate not above the base's 35.76 % and expected under
  32 %; word test at least 60 of 240.
- Singer: the comparison's verdict against the base is not WORSE (no guard broken).
- Bench within 5 % of the base, native and wasm.
- `consonant_artifacts`' existing tests still pass (the /p/ of "sharp", the /f/ click).

## Step 2. The reference solver, on still postures

1. `acoustics3d::grid`: cells from the laminae with air fractions and face openings, tissue
   per wall cell, the outside region on a spherical head, an absorbing layer; only air cells
   stored. Additions to `tract3d` (small, additive): the lips' front surface, the tissue at
   a point.
2. `acoustics3d::solver`: the leapfrog step with the stability floor, wall losses, yielding
   walls, the 1D ends (trachea, nose, glottal port); `acoustics3d::response`: transfer
   function and impulse response of a posture; `chimera acoustics3d response`.
3. `chimera acoustics3d validate`: the measured prints; `chimera acoustics3d audit`: the
   model against the MRI meshes through the solver.

**Acceptance.**

| Check | Line |
|---|---|
| Closed-open tube, 17 cm, 2 mm | first four resonances within 0.5 % of analytic (the whole-voxel prototype: 0.3 %) |
| Two cavities joined by a neck of 0.2 cm² (the case whole voxels fail), 2 mm | the neck's resonance within 2 % of the chain-matrix value |
| Tube with a closed side branch of 2 cm, 2 mm | the zero within 2 % |
| Convergence, 2 mm against 1 mm, presets /i/, /u/, /s/ (rigid) | F1–F3 within 2 % (the prototype: 12 % on /i/ F1 between those grids and the limit) |
| The 44 DVTD prints, rigid, at the prints' speed of sound | F1, F2, F3: median absolute error ≤ 2.5 % each, none above 6 %; curve difference ≤ 2 dB RMS (median) over 0.1–1 kHz and ≤ 4 dB over 1–5 kHz; 5–10 kHz reported. For scale: the 1D chain has 4.7 %, 5.1 % and 2.5 dB |
| Energy | a lossless closed box holds its energy within 0.1 % over 10⁵ steps; no growth with cut cells at the floor |
| Speed, one posture's response at 2 mm | measured and reported; ≤ 5 s on the design machine |
| The audit | per DVTD sound: the fitted model's F1–F3 against the mesh's, and every pole–zero pair below 4 kHz present in one and absent in the other; no pass line (it judges shapes owned elsewhere); the list goes to results.md and to the geometry's owner, and settles how the real-time airway treats the space beside the tongue (design.md §7) |

If the prints' line is missed, the cause is found before the solver judges anything.

## Step 3. The real-time voice from the 3D geometry

1. `tract3d` additions: cuts at given stations with the perimeter by tissue and the side
   regions' descriptors; a `compute` that can be advanced in slices.
2. `acoustics3d::reduce` and `HumanAirway`: areas by the two-rate scheme; walls by the cut's
   perimeter and tissue; side branches (piriform fossae, valleculae, the space under a
   raised tip, the /l/ pocket with its side channels, the space beside the tongue by the
   audit's verdict).
3. Corrections fitted to the solver where a gap is systematic
   (`chimera calibrate acoustics3d` → `acoustics3d.generated.json`).
4. Vowel and consonant tables regenerated (`chimera calibrate vowels all`, `consonants`).
5. The lab's 3D-only parameters reach the voice: one additive engine command for the shape
   offsets (a value area like the articulators'; no change to the gesture record), wired
   where the lab sets its posture. Every other mode already speaks through `HumanVoice`.

**Acceptance.**

| Check | Line |
|---|---|
| Areas the voice uses against 3D cuts computed at every control step, 20 training sentences × 2 bodies | sections above 0.3 cm²: median error ≤ 3 %, 95th percentile ≤ 10 %; closing and opening instants (area through 0.01 cm²) within 3 ms median, 6 ms worst |
| Continuity | the sweep of `geometry_continuity` on the areas the voice uses: no jump above that test's limit |
| Real-time airway against the solver: 16 presets and 24 postures from the training sentences, both bodies, yielding walls in both | F1–F3: median absolute error ≤ 3 %, none above 8 %; third-octave level difference over 0.2–4 kHz ≤ 3 dB RMS (median over postures); 4–10 kHz reported; and better than today's airway against the same solver on each of these figures |
| Calibrated vowels | median formant error ≤ 1 % for F1 and F2, both bodies (today 0.9 % and 0.8 % male) |
| Contract | no allocation while rendering speech through the 3D path; identical samples for block sizes 1 to 48 000; creatures' goldens bit-identical |
| Load | wasm at most 1.25 × the base measured back to back; no 128-sample block above half its duration in that bench |
| Sound | the step-1 lines hold again after recalibration; speech judge not worse than at the end of step 1; singer verdict not WORSE |
| Modes | `bun tools/e2e.ts` passes; the lab's test hears a change when a 3D-only offset changes |

## Step 4. The consonants' place and spectra from the 3D geometry

1. `acoustics3d::jets`: for each narrow place the channel's exit, the jet's width, the
   surface it meets and at what distance, the cavity in front; the sources of step 1b read
   these.
2. The /l/ and /ɹ/ branches and the mouth cavity behind a nasal closure, checked in the
   airway's transfer function and in the solver.
3. Only if a line is missed and the solver shows the tuned preset's shape would pass: the
   missing shape as a control key, with the gesture record changed in both
   `crates/chimera-wasm/src/lib.rs` and `src/wasm/wire.ts` and `LAYOUT_VERSION` bumped.

**Acceptance.** Every line of consonants.md §9, both voices, on the 3D path: F3 (/s/ at
4–8 kHz, /ʃ/ at 2.5–4 kHz, a factor of 1.4 apart), L1, N1 and the lines of step 1 again.
The /k/ burst's spectrum has one peak within 20 % of the following vowel's F2–F3 region
(1.5–4 kHz), the /t/ burst's centre of gravity lies above 3.5 kHz, the /p/ burst's below
both.

## Step 5. The precise render

1. `acoustics3d::airway3d`: the solver's cells as the voice's pharynx and mouth, with moving
   fractions, the glottal port, the nose branch, the walls, the noise sources on faces.
2. `chimera render precise <out> "text" [body] [--grid mm] [--compare]`.

**Acceptance.**

| Check | Line |
|---|---|
| A sustained vowel through the precise voice against the posture's response times the same source | third-octave levels within 1 dB, 0.1–8 kHz |
| The battery and the 20 training sentences, both bodies, 2 mm | finite output, no sample-to-sample step above the largest in the real-time render of the same phrase by more than 6 dB, closures silent to −50 dB re the vowel |
| The operator's phrase and five training sentences, precise against real time | the consonant lines measured on the precise renders (reported, with any that fail); band levels of the difference per octave in results.md |
| Speed | measured; ≤ 60 × real time on one core of the design machine at 2 mm |

## Step 6. Speed and the contract

Native and wasm benches against the base, back to back, three runs each; the guard of step 3
holds at the end. If it does not: a fast path in the human wave step for branches without
kinetic loss, with coefficients at control rate (creatures untouched, their goldens
bit-identical). The allocation test, the block-size test and the native-against-wasm identity
test pass.

## Step 7. The precise render in the app (optional)

Taken only if the phrase "Hello, can you hear me?" renders at 3 mm in under 30 s in Chromium
on the design machine. A worker runs the same code; one action in the Speak mode starts it,
shows progress and plays the result. Layout lint and e2e cover it.

## Step 8. The judges, the comparison, the documents, the checks

| What | Line |
|---|---|
| Speech judge, training sentences | ≤ 30.0 % (base 35.76 %), verdict BETTER by `--against` the base |
| Speech judge, held-out | ≤ 37.0 % (base 42.0 %); reported, never tuned to |
| Operator's phrase | 0 of 5 words wrong, both voices, and K1–K4 |
| Word test | ≥ 84 of 240 (35 %; base 28); per-consonant table in results.md |
| Singer | verdict not WORSE against the base; every guard's number in results.md |
| Before and after | the operator's phrase, six training sentences and the battery, both bodies: the base's real-time render, the final real-time render, the final precise render; spectrograms; a written comparison in results.md (what changed in the consonants, the breath noise, the vowels after stops, the response above 4 kHz) |
| Documents | results.md; AGENTS.md's map, rules and checks table gain `acoustics3d` and its commands; `docs/architecture.md` and `docs/rust-engine.md` say what changed; `docs/tract3d/README.md` points here for the sound |
| Checks | `bun run test`, `bun run typecheck`, `bun run lint:layout` (run alone; it takes about 35 minutes and wants no other headless browser beside it), `bun run build && bun run preview` then `bun tools/e2e.ts` |

## If the lane runs long: what to cut, in this order

1. Step 7 (the app's action).
2. From step 4: the spectral lines of liquids and nasals (L1, N1) become reported values.
3. From step 5: whole phrases at 1 mm, and the precise-against-real-time table beyond the
   operator's phrase.
4. From step 3: the lab's shape offsets reaching the voice.

Never cut: step 1 (the consonants and "can"), step 2's validation of the solver, step 3's
voice from the 3D geometry with its accuracy table, step 5's renderer for at least the
operator's phrase, step 8's judges and checks. If one of these cannot be met, the lane stops
and reports which line, with its number.

## Decisions that belong to the operator, if they arise

- A natural-range consonant line and a singer guard cannot both hold (the hill climb met
  this with louder sibilants and note MCD): which gives way.
- The audit finds the 3D shapes acoustically wrong in ways the real-time airway cannot work
  round: whether this lane may change the shapes.
