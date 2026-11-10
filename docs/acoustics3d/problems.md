# Problems: what today's voice gets wrong, measured

*Design stage, 2026-11-10, base revision 6a0602d. The requirement is quoted at the top of
[design.md](design.md). Every number here was measured on this revision; the renders, traces,
scripts and frames are outside git under `$LLV_EVIDENCE_DIR/acoustics3d/design/` (its
`README.md` lists the folders). Natural-speech ranges and their sources are in
[consonants.md](consonants.md).*

## 1. How it was measured

- **The engine's own renders.** `chimera render text|speech` at 48 kHz, male and female bodies.
- **A probe** (`scripts/probe`, a scratch crate over `chimera`): it plans a phrase with the
  engine's planner, renders it one sample at a time and logs the engine's state every control
  step (64 output samples): the narrowest area and where it is, the pressures below the
  glottis, in the pharynx and behind the constriction, the glottal area and flow, the folds'
  vibration gate, the flow through the constriction, the level of each noise source, the
  radiated flows. Its audio equals the CLI's byte for byte (checked on the operator's phrase,
  both bodies).
- **A battery**: each of 24 English consonants in /ˈɑCɑ/, /əˈCɑ/, /ˈCɑ/, /ˈɑC/ and /ˈiCi/, both
  bodies (230 tokens), and the 20 training sentences of `tools/asr/sentences.tsv` (632
  consonants). `scripts/analyze.py` measures each consonant against its neighbouring vowel.
- **Definitions.** *Vowel level*: RMS of the loudest 30 ms inside the reference vowel's planned
  span (the following vowel when there is one, else the preceding). *Closure*: the engine's
  narrowest area at or below 0.01 cm². *Burst*: the loudest 5 ms starting within −3…+25 ms of
  the release; also above 1.5 kHz, where a burst stands clear of voicing. *VOT*: release to
  the folds' vibration gate passing one half. *Aspiration*: RMS from 15 ms after the release
  to 5 ms before voicing. *Fricative level*: RMS of the middle 60 % of the planned span.
- **The project's speech judge** (`tools/asr/setup.sh`, `speech_wer.py`), unchanged, and two
  scripts that reuse its recogniser and settings: `transcribe.py` (any WAV) and `wordtest.py`
  (minimal-pair words in "Say … again.").
- **Ablations.** Scratch-only switches in a copy of the engine (`scripts/ablation-engine.patch`)
  that turn one suspected cause off or scale it. With every switch off the copy renders the
  base audio byte for byte. They test causes only; the proposed changes are in consonants.md.
- **Machine.** Intel i9-12900K, Linux; other work ran on it throughout (load average 8–14),
  so timings are best-of-several on one pinned P-core thread and say so.

## 2. The phrase that was reported: "Hello, can you hear me?"

The planner's segments for the second phrase: /k/ 0.797–0.877 s, /ə/ 0.877–0.943, /n/
0.943–0.999. What the engine does (male; female in brackets):

| What | Measured | Natural speech |
|---|---|---|
| Closure | 91 ms (89), intraoral pressure 6.7 cmH2O at the release | 50–100 ms, 5–8 cmH2O |
| Burst, whole band, re the vowel of "you" | −4.8 dB (−12.6) | — |
| Burst above 1.5 kHz, re that vowel | −15.4 dB (−23.4) | a velar burst is the longest and most compact of the three places |
| Frication at the constriction after the release | the source runs for 13 ms (15), about 10 dB under the transient (§5.1) | 20–30 ms of velar frication |
| VOT | 68 ms (68) | about 30–45 ms for /k/ in running speech; 70–80 ms before a stressed vowel in a word said alone (consonants.md §2) |
| The vowel /ə/ (planned 66 ms) | the folds vibrate for **0 %** of it; voicing starts 1 ms after the /n/ begins | voiced for most of its length |
| A thump 50 ms before the closure | −4.9 dB (−13.4) re the vowel: as loud as the burst (−0.1 dB, −0.8 dB) | none |

So "can" is rendered as a thump, a silence, a short click with weak breath noise, then /n/: no
vowel, and a /k/ whose only cue is a transient 15–23 dB under the vowels above 1.5 kHz.
Frames: `base/spectrograms/hello-male.png` (the whole phrase; the /k/ is the thin line at
0.88 s), `frames/01-can-base-male.png` (0.62–1.12 s: the thump at 0.12–0.20 s of the frame,
the burst at 0.255 s, voicing only from 0.325 s, where the nasal murmur starts).

Whisper transcribes the phrase correctly in both voices (`base/operator-phrase-asr.json`): the
words are predictable, so the judge cannot see this defect. The acoustic measures above are
what an acceptance test must use for it.

With five scratch switches together (the breath starting after the closure, a shorter
glottal opening before a short vowel, the constriction source ten times stronger and kept on
to 2 cm², the dropped source restored, the glottal noise three times stronger), the same
phrase has VOT 29 ms, the /ə/ voiced for 57 % of its span, and the burst above 1.5 kHz at
−6.0 dB (male) and −12.4 dB (female) (`ablation/can/`, `frames/02-can-ablation-male.png`).
That confirms which parts are missing; the thump is untouched by all five (§5.9).

## 3. Intelligibility at the base

| Measure | Base |
|---|---|
| Speech judge, training sentences (word error rate) | **35.76 %** (Harvard male 28.75, female 33.75; nonsense male 42.25, female 39.44) |
| Speech judge, held-out sentences | **42.0 %** (Harvard 30.38 / 30.38; nonsense 52.11 / 57.75) |
| Training sentences with at least one error | 33 of 40 |
| Operator's phrase | 0 of 5 words wrong, both voices |
| Word test, "Say … again." (120 minimal-pair words × 2 voices) | **28 of 240 right (11.7 %)**: onsets 10/88 male, 14/88 female; codas 4/32 and 0/32 |

The word test is strict (the whole word must be written) and mixes vowel and coda errors into
an onset's score, so it is an indicator beside the judge. Its errors agree with the acoustic
tables: no /k/, /s/, /tʃ/, /dʒ/, /v/, /z/, /θ/ or /ɹ/ onset was written right in either
voice; "say" is often written "shape", "shake" or "she" (the /s/ is heard as /ʃ/, §5.7).
The hill-climb log's finding stands that the sentence judge moves by about ±2 points on
inaudible changes (`docs/hill-climb/log.md`, experiment 20).

## 4. The consonants against natural speech

Stressed onsets (mean of /əˈCɑ/ and /ˈCɑ/), male / female. Levels are dB re the vowel.

| C | Closure ms | Intraoral cmH2O | Opening cm²/s | Burst | Burst > 1.5 kHz | Within 10 dB of its peak, ms | VOT ms | Aspiration |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| p | 98 / 97 | 4.6 / 5.4 | 108 / 86 | −8.6 / −16.7 | −21.9 / −32.7 | 8 / 8 | 63 / 63 | −20.4 / −30.3 |
| t | 77 / 77 | 7.2 / 7.5 | 84 / 69 | −8.1 / −17.9 | −11.0 / −20.6 | 10 / 9 | 63 / 62 | −17.9 / −28.2 |
| k | 87 / 85 | 8.1 / 8.1 | 77 / 62 | −10.7 / −19.8 | −16.7 / −25.0 | 17 / 14 | 65 / 64 | −18.2 / −29.0 |
| b | 99 / 98 | **−2.8 / −0.7** | 117 / 93 | (voiced) | −23.6 / −16.2 | – | 0 / 0 | – |
| d | 76 / 77 | 4.3 / 5.0 | 83 / 67 | (voiced) | −19.3 / −15.7 | – | 1 / 3 | – |
| g | 86 / 85 | 7.7 / 7.7 | 71 / 57 | (voiced) | −18.2 / −16.1 | – | 8 / 9 | – |
| tʃ | 36 / 36 | 5.2 / 5.8 | 13 / 10 | −13.0 / −22.3 | −13.0 / −22.4 | 82 / 45 | 99 / 99 | −17.1 / −26.6 |
| dʒ | 27 / 27 | 1.6 / 2.5 | 12 / 9 | (voiced) | −24.7 / −25.7 | – | 0 / 0 | – |

| C | Level | Spectral peak Hz | Centre of gravity Hz | Share above 2.5 kHz | Area cm² | Flow cm³/s | Intraoral cmH2O |
|---|---:|---:|---:|---:|---:|---:|---:|
| f | −23.2 / −34.2 | 9979 / 8150 | 7814 / 7498 | 0.95 / 0.94 | 0.16 / 0.13 | 278 / 251 | 2.3 / 2.4 |
| θ | −25.8 / −36.5 | 10072 / 10271 | 9048 / 9456 | 0.99 / 0.98 | 0.16 / 0.13 | 336 / 299 | 3.1 / 3.1 |
| s | −14.1 / −25.3 | 3434 / 3855 | 4063 / 4681 | 0.98 / 0.99 | 0.10 / 0.08 | 270 / 214 | 4.1 / 4.4 |
| ʃ | −16.7 / −27.7 | 2279 / 2490 | 3900 / 3762 | 0.44 / 0.64 | 0.22 / 0.18 | 442 / 358 | 2.6 / 2.6 |
| v | −7.8 / −9.8 | 457 / 410 | 640 / 577 | **0.02 / 0.01** | 0.28 / 0.26 | 8 / 61 | 0.3 / 0.3 |
| ð | −10.3 / −14.8 | 340 / 422 | 517 / 467 | **0.02 / 0.00** | 0.21 / 0.17 | 128 / 152 | 0.9 / 0.9 |
| z | −12.7 / −17.8 | 334 / 404 | 2277 / 1113 | 0.47 / 0.16 | 0.13 / 0.11 | 200 / 166 | 1.5 / 1.6 |
| ʒ | −10.0 / −13.6 | 340 / 410 | 603 / 528 | **0.03 / 0.01** | 0.26 / 0.22 | 227 / 223 | 0.7 / 0.7 |
| h | −23.0 / −33.9 | 1535 / 1822 | 6140 / 5012 | 0.71 / 0.62 | 0.95 / 0.77 | 655 / 531 | 0.2 / 0.2 |

(The pressures and flows in this table are medians over all contexts.) Nasals, liquids and
glides sit 2–13 dB under the vowel in both voices (m −8.6 / −13.1, n −9.2 / −12.6, ŋ −2.4 /
−5.6, l −6.3 / −8.7, ɹ −3.9 / −5.1, w −7.0 / −7.9, j −11.1 / −13.2), inside the natural
range; their spectra (the /ɹ/'s third formant, the nasal zeros, the /l/'s zero) were not
measured in this stage. The full tables for all five contexts are `base/battery.md`.

In running speech (the 20 training sentences) the same picture holds: voiceless stop bursts
above 1.5 kHz at −21 dB (/p/), −13 dB (/t/), −17 dB (/k/) male and −30, −21, −26 dB female;
/s/ −13.4 dB male and −22.1 dB female; /ʃ/ −14.5 and −22.3; /f/ −20.4 and −29.6; /h/ −21.0
and −29.6 (`base/sentences.json`).

**Vowels after a voiceless stop** (running speech, both voices alike): a stressed vowel is
voiced for a median 45 % of its planned span (5 of 10 under half); an unstressed one for
12–14 % (all 7 under half, 3 with no voicing at all).

Against the natural ranges of consonants.md:

- voiceless stops: /p/ and /k/ bursts carry almost nothing above 1.5 kHz, the burst lasts
  8–17 ms at every place, VOT is one value (62–65 ms) at every place and in every context,
  aspiration is 18–30 dB under the vowel;
- voiced stops: no audible burst, and /b/ is released on a negative intraoral pressure;
- sibilants: 14–17 dB under the vowel (male) and 25–28 dB (female), with spectral peaks at
  3.4–3.9 kHz for /s/ and 2.3–2.5 kHz for /ʃ/ where natural ones lie at 4–8 and 2.5–4 kHz;
- voiced fricatives /v ð ʒ/ and the release of /dʒ/: 0–3 % of their energy above 2.5 kHz,
  so they are vowel-like sounds with no frication (frame `frames/07-battery-zh-male.png`);
- affricates: a closure of 27–36 ms that often does not seal before /ɑ/ (narrowest area
  0.003–0.004 cm²);
- the female voice: every noise 9–11 dB lower against her vowels than the male's
  (frame `frames/06-battery-sh-female.png`).

## 5. Why: the causes, each with its evidence

### 5.1 A stop's release has no frication burst

The audible burst is the pressure-release transient alone. The constriction's own turbulence
source contributes nothing:

- **It lies about 10 dB under the transient.** With the dropped source restored (below),
  four times its gain moves the /k/ burst above 1.5 kHz by 0.4–0.7 dB (male) and 2.4–4.2 dB
  (female); ten times, by 6–8 dB (male, −18.2 → −10.2 dB in /əˈkɑ/) and 10–12 dB (female,
  −31.5 → −19.5 dB in /ˈkɑ/); the /p/ burst moves by 0.1–0.5 dB (male) even at ten times
  (`ablation/gains/`). The gain is `NoiseGains.constriction` 2.2·10⁻⁶
  (`voice/human_airway.rs:149`), and the source is band-limited to 350–2600 Hz (line 330).
- **It is on for 5–15 ms.** Its weight fades to zero between 0.35 and 0.9 cm² of opening
  (`human_airway.rs:309`), and the constriction opens at 60–120 cm²/s (§5.3), so the jet has
  a source for four to eleven control steps.
- **For velars before back vowels it is dropped.** `update_noise` maps a section index to a
  branch, and the first oral section maps to "no branch" (`human_airway.rs:318`). A
  constriction on that section, or two sections on either side of it (its injection points),
  loses both the constriction source and the teeth source. The velar closure before /ɑ/ sits
  there (male: section 20 of 36, with 18 pharyngeal sections): the trace shows flows of up to
  815 cm³/s through an opening of 0.03–0.93 cm² with the source's level at zero
  (`base/battery/k-male.json`, 0.339–0.351 s). Restoring the branch turns the source on for
  12 ms and changes the burst by 0.0–0.4 dB (`ablation/junction/`), which is the first point
  again.
- **/t/ is the exception**: its release fires the teeth (obstacle) source, which is why /t/
  has the strongest burst above 1.5 kHz (−11 dB male). Scaling that source by 4 puts the
  /t/ burst at +0.5 dB re the vowel.

### 5.2 One voice onset time, longer than short vowels

The planner opens the glottis for an aspirated stop with one gesture
(`phonetics/planner.rs:312–314`): abduction 0.36 cm² until 20 ms after the release, then a
45 ms ramp. The folds resume once the opening falls under about 0.15 cm²
(`sources/glottis.rs:173`). The result is 62–68 ms at every place, in stressed and
unstressed syllables, before long and short vowels. An unstressed vowel of 60–90 ms has no
voicing left ("can", "to", "the … of"). The ablation that closes the folds earlier before
vowels under 110 ms gives the /ə/ of "can" 57 % voicing and −1 dB re the neighbouring vowel
(from −9 dB).

### 5.3 Every place opens at the same speed

The area of a constriction may grow by `(45 + 40·A)·k²` cm²/s (`human_airway.rs:150–151`)
whatever the articulator. Measured over the first 5 ms: /p/ 73–120, /t/ 54–87, /k/ 19–78
cm²/s (the slow velars are the /i/ context and the final position). Published estimates are
about 100 cm²/s for lips and tongue tip and about 25 cm²/s for the tongue body (consonants.md
§2). A velar that opens three times too fast has no time for its long burst.

### 5.4 The breath pressure collapses at consonant airflows

The lungs feed the trachea through a port whose resistance is half the trachea's
characteristic impedance, 8.0 dyn·s/cm⁵ (`human_airway.rs:108`): an absorbing end for sound,
used as the DC resistance too. At the 650–700 cm³/s of an open glottis it drops 5.5 of the
8 cmH2O: the trace of "can" shows 2.3–2.4 cmH2O below the glottis during the aspiration, and
the fricatives hold 2.3–4.4 cmH2O in the mouth where speakers hold 5–8. A healthy airway's
resistance is 1–2 cmH2O per litre per second (consonants.md §1). With the resistance at
1.76 dyn·s/cm⁵ (`ablation/lung/`): aspiration +3.4…+4.1 dB, /h/ +4.4…+5.0 dB, /f/ +3.1…+3.3
dB, /ʃ/ +2.5 dB, /s/ +1.4…+1.7 dB, the mouth pressure of /s/ 4.3 → 5.9 cmH2O.

### 5.5 Aspiration and /h/ are band-limited and weak

The glottal noise source is a 300–3500 Hz band (`human_airway.rs:294`) with gain 3.2·10⁻⁶.
Aspiration is 18–20 dB under the vowel (male) and 28–30 dB (female); three times the gain
brings the male /p/ to −12 dB. The same source is what should fill the space between a
vowel's harmonics above 3–4 kHz, and it has no energy there (§6).

### 5.6 The female voice's consonants are 9–11 dB lower

Already traced in the hill-climb log (experiment 13): her vowels are about 5 dB too loud for
her folds (the pulse area scales with k², her output gain is a level match), and the
turbulence law (amplitude ∝ Re² − Re_c², `airway/network.rs:1220–1223`) loses about 3–5 dB
with the body's scale. This stage's tables repeat the size of the gap on every noise class.

### 5.7 The sibilants are low in level and in pitch

The teeth source is white between 1.2 and 5 kHz (`human_airway.rs:341`), so /s/ peaks at
3.4–3.9 kHz and /ʃ/ at 2.3–2.5 kHz, a little over half an octave apart, both lower than
natural ones; the judge writes "say" as "shape" or "she". Their level against /f/ is 9 dB
(male /s/ over /f/), near the natural 10–15 dB, so the sibilants' own level is the smaller
problem; the pressure of §5.4 and the female gap of §5.6 are the larger ones. The 1D tract
also gives the cavity in front of the constriction one resonance where the real one, with
the lips and the space under the tongue, has several (design.md §3).

### 5.8 Voiced fricatives and voiced stops have too little pressure

- **Voiced fricatives**: the mouth holds 0.3–1.5 cmH2O (/v/ 0.3, /ʒ/ 0.7, /ð/ 0.9, /z/ 1.5),
  the jet runs at 6–16 m/s (voiceless: 20–28 m/s) and its Reynolds number sits at 2000–4100,
  just over the source's threshold of 1800, so the noise is 15–20 dB under the voiceless
  one. The planner's opening for them is 0.035 cm² (`planner.rs:319`) and their constrictions
  are 0.21–0.29 cm² for /v ð ʒ/.
- **Voiced stops**: the articulators keep moving towards the next vowel while the mouth is
  shut, the folds pass 10–100 cm³/s, and the growing cavity takes more than that: /b/ is
  released at −2.8 cmH2O (male). Nothing drives a burst.

### 5.9 A thump before a phrase-initial closure

50 ms before the /k/ of "can" closes, a transient radiates at −4.9 dB re the vowel (male),
with the glottis shut (0.015 cm²) and no lung pressure yet; the pharynx sits at +0.22…+0.25
cmH2O for 16 ms. It is the air the tongue and jaw push out on their way to the closure (the
moving-wall volume source): with that source off the thump falls by 12 dB (−4.9 → −16.9 dB
male, −13.4 → −19.0 dB female, `ablation/can/5-novolume-*`). Moving the breath's start to
after the closure does not change it (−4.9 → −4.9 dB). In the battery's utterance-initial
tokens the loudest 5 ms before the first segment is −26 dB (stops, median), up to −15 dB;
before sonorants −9…−15 dB. Why the displaced flow starts fast enough to be heard (the
articulators' start, the area limiter or the source's 60 Hz smoothing) was not isolated.

### 5.10 Smaller things seen on the way

- **A hiss before the /n/ of "can".** The tongue tip closes for /n/ while the folds are still
  apart (0.24–0.29 cm²): the teeth source fires for 21–24 ms from 0.905 s. It goes away with
  §5.2.
- **Affricates** aim their stop part at the fricative's place for 55 % of a short segment and
  seal for 27–36 ms, or not at all before /ɑ/.
- **Radiation and closure timing were checked and are sound.** The lips' and nostrils'
  loads pass the release transient as designed (the output is the time derivative of the
  radiated flow); the missing part lies upstream, in the sources (§5.1–5.5). The planner's
  closures and releases land within a few ms of the plan (closures of 76–99 ms for planned
  segments of 60–85 ms with their leads).

## 6. Why it sounds synthetic: what could be measured

| Measure | Base | Reference |
|---|---|---|
| Depth between a vowel's harmonics and the valleys between them, 4–8 kHz (64 ms window, 63 + 62 fully voiced vowels of the training sentences) | 8.5 dB male, **21.5 dB female** (2–4 kHz: 17.7 and 32.0 dB) | a natural voice's harmonics sink into breath noise above 3–4 kHz (consonants.md §4); no like-for-like natural recording was measured here |
| Long-term spectrum re 500 Hz (`chimera diagnose`) | 1 kHz −10.7 dB, 2 kHz −17.6 dB | the tool's norms: −6 and −11 dB |
| Voiced time on the flat 110 Hz baseline (`chimera diagnose`) | 29.3 %; range 6.0 semitones | — |
| Vowels after voiceless stops | §4: mostly voiceless | voiced |
| Resonances against a 3D computation of the same geometry | design.md §3: the first three resonances differ by 3–15 %, and above 4 kHz the 3D response has zeros the tube model has no way to make | — |

The voice is periodic far above the range where a human voice turns to noise, most of all the
female one; its consonants are faint or missing, so speech reads as a chain of vowels; short
vowels after /p t k/ are whispered; and the tract's response above 4 kHz is the regular comb
of a tube. Pitch movement is also stylised (29 % of voiced time on one flat line); that is
the planner's intonation and is outside this lane (design.md, deferred).

## 7. Not established

- No natural recording was run through these scripts, so "natural" is the published ranges of
  consonants.md, whose definitions are close to ours and differ in detail.
- The spectra of nasals, /l/ and /ɹ/ were not measured.
- Where the thump of §5.9 gets its speed.
- The singing judge (the CSD comparison) was not run: its dataset is not on this machine
  (`tools/fetch-csd.sh` fetches it); plan.md has it as the first step.
- The wasm load was measured in Bun and Node (design.md §3); a browser (Chromium) is still to be timed.
