# Consonants: physics, parameters, natural ranges

*Design stage, 2026-11-10, base 6a0602d. What each consonant class needs from the air supply,
the glottis, the constriction and the planner; what the engine has today
([problems.md](problems.md)); what changes; and the ranges of natural speech that the result
is measured against. The measures are those of problems.md §1, taken with the same scripts
on the same battery and sentences, so "before" and "after" are like for like.*

**On the sources.** Entries marked † were read on 2026-11-10 in the paper itself or in a
paper that reports them; ‡ in a summary of the paper found by search that day. Unmarked ones
are quoted as the literature commonly gives them. Both ‡ and unmarked entries must be
re-read at the source before a range is tightened; the ranges below are set wide enough to
hold either way. Published definitions differ a little from ours (which window,
which part of the vowel), so a range is a guide to the size of an effect; the acceptance
lines of §9 are this project's own.

## 1. The air supply

**Physics.** Speech runs on a nearly constant subglottal pressure of 5–10 cmH2O (about 6–8 at
normal loudness; Holmberg, Hillman & Perkell 1988). The lower airways' resistance is 1–2
cmH2O per litre per second (respiratory physiology texts), so even the 1 l/s of an aspirated
release costs 1–2 cmH2O. Behind a voiceless closure or fricative the mouth pressure climbs to
almost the subglottal value: 6–7 cmH2O for /p t k/ and /s/, 4–5 for voiced stops and
fricatives (Subtelny, Worth & Sakuda 1966; Arkebauer, Hixon & Hardy 1967). Flows: 200–500
cm³/s through a fricative, a peak near 1 l/s just after an aspirated release and in /h/
(Klatt, Stevens & Mead 1968; Stevens 1998, ch. 2).

**Today.** The lungs are a pressure node behind a port of 8.0 dyn·s/cm⁵
(`voice/human_airway.rs:108`), half the trachea's characteristic impedance: right as an
absorbing end for sound, four to five times a real airway's resistance for steady flow. The
subglottal pressure falls to 2.3–2.4 cmH2O during aspiration and the fricatives hold 2.3–4.4
cmH2O (problems.md §5.4).

**Change.** A lung termination that is absorbing for sound and has the airway's own
resistance for steady flow: the port's resistance in parallel with an inertance, in series
with 1.5–2 dyn·s/cm⁵, the corner between 30 and 80 Hz (below the voice's fundamental, above
the breath's changes). Parameters: the DC resistance (1.5–2.0), the corner frequency. The
subglottal resonances keep their damping; the builder checks them with the transfer-function
test of `golden_analysis`.

## 2. Stops: /p t k b d g/

**Physics, in order** (Stevens 1998, ch. 7; Fant 1960):

1. *Closure.* The articulator seals; contact spreads, so the last part of closing is slower
   than opening. Voiceless: the glottis opens during the closure, the mouth pressure reaches
   the subglottal pressure within 20–40 ms. Voiced: the folds keep vibrating as long as the
   pressure across them stays above 2–3 cmH2O; the walls yield and the larynx lowers to make
   room, and the mouth pressure rises to 4–5 cmH2O by the release. The walls radiate the
   voice bar.
2. *Release, transient.* The seal breaks; the pressure behind it discharges through a growing
   slit. The first millisecond is a pulse of volume velocity (a monopole): all the model has
   today.
3. *Release, frication burst.* While the slit is under about 0.2–0.3 cm² the jet through it
   is turbulent; its noise is shaped by the cavity in front of the constriction. Lips: no
   cavity in front, a weak burst with a flat or falling spectrum, 5–10 ms. Tongue tip: a
   short front cavity, energy above 3.5–4 kHz, 10–20 ms. Tongue body: a long front cavity,
   one compact peak near the following vowel's F2–F3 (1.5–4 kHz), 20–30 ms, often double.
   The delay between the first opening and the final opening grows from lips to tip to body
   (Hanson & Stevens 2000 †), because the body is heavy and peels off the palate.
4. *Aspiration.* With the glottis still open, the flow (up to 1 l/s) makes noise at the
   glottis and the false folds, exciting every formant from F2 up; F1 is damped by the open
   glottis. For /t/ and /k/, frication at the consonant's place often continues under the
   aspiration (Hanson & Stevens 2003 †).
5. *Voice onset.* The folds come together and start; the first cycles are breathy.

**Natural ranges.**

| Quantity | Range | Source |
|---|---|---|
| Closure, between vowels | 50–100 ms | Crystal & House 1988; Byrd 1993 |
| Mouth pressure at the release | 6–7 cmH2O voiceless, 4–5 voiced | Subtelny et al. 1966; Arkebauer et al. 1967 |
| Area growth at the release | lips and tip about 100 cm²/s by estimate, 20–50 measured on voiceless stops; tongue body about 25. A synthesis study tested 15–100 for /b d/ and 10–50 for /g/; listeners preferred the faster rates for the lips | Massey 1994, and later studies of release rate reported at ICPhS 1995 ‡ |
| Burst level, alveolar | within 14–15 dB of the vowel's RMS for American English /t d/ (vowel-to-burst amplitude ratio under 5) | Jongman, Blumstein & Lahiri 1985, as reported by Sundara 2005 † |
| Burst level by place | alveolar and velar strong, labial weakest | Zue 1976; Stevens 1998 |
| VOT, words said alone | /p t k/ 58, 70, 80 ms; /b d g/ 1, 5, 21 ms | Lisker & Abramson 1964 |
| VOT, stressed, in a sentence frame | /p t k/ 47, 65, 70 ms; /b d g/ 11, 17, 27 ms; after /s/ 12, 23, 30 ms | Klatt 1975 |
| VOT, running speech | shorter: about 30–45 ms for /p t k/ | Lisker & Abramson 1967 |

**Today → change.**

| Part | Today (problems.md) | Change | Parameters |
|---|---|---|---|
| Opening speed | one limit for every place, 54–120 cm²/s measured | the limit belongs to the articulator that makes the constriction: lips and tongue tip fast, tongue body slow, with the slow first phase of a peeling contact | per articulator class: initial rate (cm²/s), the area up to which it holds |
| Frication burst | about 10 dB under the transient, on for 5–15 ms, dropped for velars before back vowels | the jet's source of §3, on for as long as the jet is turbulent (by its own Reynolds number, no area window), at every position including the junction of the two tubes; its obstacle is whatever the jet meets (teeth for /t/, the palate and front cavity walls for /k/, nothing for /p/) | the source law's constants (§3) |
| Aspiration | 300–3500 Hz band, 18–30 dB under the vowel | §4 | §4 |
| VOT | one gesture: 62–68 ms everywhere | the glottal opening's size and timing by place, stress and what follows: the opening peaks at the release; the folds are back together a set time after it (lips shortest, body longest), sooner in unstressed syllables and function words, and never later than 40 % into the following vowel; unaspirated after /s/ | target VOT per place (stressed: 55, 65, 75 ms), factor for unstressed (0.5–0.6), share of the vowel (0.4) |
| Voiced stops | released at −2.8…+8 cmH2O, no burst | the vowel's jaw and tongue gestures wait for the release where they would enlarge the closed cavity; the folds' opening widens a little during the closure (as real speakers do to keep voicing) so the mouth reaches 2–5 cmH2O; the same burst source as voiceless stops, at its lower pressure | the abduction during a voiced closure, the hold on cavity-enlarging gestures |
| Before a phrase-initial closure | a thump as loud as the burst | the displaced air's flow may not start faster than the articulators can (problems.md §5.9: find which stage adds the speed, then fix that stage); the breath starts once the closure is made | found by test |

## 3. Fricatives: /f θ s ʃ v ð z ʒ/

**Physics.** A jet leaves a constriction of 0.05–0.3 cm² at 20–40 m/s. It makes sound in two
ways (Shadle 1985, 1990; Stevens 1971, 1998, ch. 2 and 8):

- *on a wall or in the free jet* (a weak, distributed source): /f θ/, /ç x/, the first
  phase of a stop burst;
- *on an obstacle* that the jet strikes face on, the lower incisors for /s ʃ/ (a strong
  dipole, a pressure source in series with the tract at the obstacle): sibilants are about
  10–15 dB more intense than /f θ/ for this reason (Behrens & Blumstein 1988; Jongman,
  Wayland & Wong 2000 ‡).

The source's sound pressure grows as U³·A^−2.5, the same as ΔP^1.5·A^0.5 (Stevens 1971):
one more power of the jet's speed than the engine's Re² law, and less dependence on area. Its
spectrum is broad, centred near a Strouhal number of 0.1–0.2 times the jet's speed over its
width, and falls above; the builder reads the spectral law in Shadle 1990 or Stevens 1998
before coding it (the hill-climb log's experiment 20 asks the same). The radiated spectrum is
the source filtered by the cavity in front of it: a short cavity for /s/ (peak 4–8 kHz), a
longer one with the space under the tongue and the rounded lips for /ʃ/ (2.5–4 kHz), almost
none for /f θ/ (flat to 8–10 kHz). Behind the constriction the tract shows up as weak zeros.
Voiced fricatives run on a lower mouth pressure (the glottis takes part of the drop), and
their noise is pulsed at the voice's rate because the jet's speed follows the glottal flow.

**Natural ranges.**

| Quantity | Range | Source |
|---|---|---|
| Spectral peak | /s z/ 6.8 kHz, /ʃ ʒ/ 3.8 kHz, /f v/ 7.7 kHz, /θ ð/ 7.5 kHz (means over men and women; men lower) | Jongman, Wayland & Wong 2000 |
| Spectral peak, /s/ and /ʃ/, other measurement | /s/ 3.5–5 kHz ‡, /ʃ/ 2.5–3.5 kHz | Behrens & Blumstein 1988 |
| Level against the vowel | sibilants about 10–15 dB over the non-sibilants ‡; voiced lower than voiceless | Jongman et al. 2000; Behrens & Blumstein 1988 |
| Noise length, word-initial before a stressed vowel | voiceless 150–180 ms, voiced 80–120 ms in words said alone; 60–110 ms in running speech | Jongman et al. 2000; Klatt 1979 (the planner's table) |
| Noise needed to tell which fricative it is | 30 ms for /ʃ z/, 50 ms for /f s v/; /θ ð/ need more † | Jongman 1989 |
| Mouth pressure | 5–7 cmH2O voiceless, 3–5 voiced | Subtelny et al. 1966; Arkebauer et al. 1967 |
| Onset of turbulence | Reynolds number about 1800 | Meyer-Eppler 1953 |

**Today → change.**

| Part | Today | Change | Parameters |
|---|---|---|---|
| Source law | amplitude ∝ Re² − Re_c², saturating (`airway/network.rs:1220`) | Stevens' law on the jet's own speed and area, with the same threshold; one law for the free jet and one, about 10–15 dB stronger, for a jet that meets an obstacle within reach | two constants (free jet, obstacle), the threshold |
| Spectrum | fixed bands: 350–2600 Hz (jet), 1.2–5 kHz (teeth), 300–3500 Hz (glottis) | a band that follows the jet (centre ∝ speed / width), wide enough to feed /s/'s 4–8 kHz and /f/'s 8 kHz; the engine's internal rate (96 kHz) carries it | Strouhal constant, bandwidth |
| Place | the narrowest section of the midline area function; the teeth at a fixed distance window | from the 3D cuts: the channel's exit (its most anterior narrow cut), the jet's width (the channel's, so a grooved /s/ and a wide /ʃ/ differ), the obstacle the jet meets (teeth, lip, palate) and the cavity in front (design.md §5.2) | none new: read from the geometry |
| Air supply | 2.3–4.4 cmH2O in the mouth | §1 | §1 |
| Female level | 9–11 dB lower than the male against the vowel | the source law above scales with pressure more than with size; her glottal pulse area by her fold length (hill-climb log, experiment 13); then her output level re-matched | fold-length factor |
| Voiced fricatives | /v ð ʒ/: no frication | a wider glottal opening with voicing kept (breathy voicing, 0.06–0.10 cm²), constrictions at the calibrated areas of their voiceless partners plus 10–20 %, so the mouth holds 3–5 cmH2O | abduction, areas |
| /f θ/ onset | within 10 dB of its peak for 15–21 ms of 100 | follows from the supply and the law; the constriction must be in place when the folds open | timing check |

## 4. Aspiration, /h/, and the breath in the voice

**Physics.** Air through an open glottis (0.2–0.4 cm² for /h/ and aspiration) is turbulent
at the glottis and where the jet meets the false folds and the epiglottis. The noise excites
the whole tract from just above the glottis; because the glottis is open the first formant is
broad and weak and the noise sits mostly in F2 and above. The same source runs in every open
phase of normal voicing: it is why a voice's harmonics give way to noise above 2–4 kHz,
earlier and more in breathy and in many female voices (Klatt & Klatt 1990; Hanson 1997).
Voiced /ɦ/ between vowels is breathy voicing.

**Today.** One source at the glottis, 300–3500 Hz, weak: aspiration at −18…−20 dB (male) and
−28…−30 dB (female), /h/ at −23 and −34 dB; a vowel's harmonics stand 8.5 dB (male) and
21.5 dB (female) clear of the valleys between them at 4–8 kHz (problems.md §6).

**Change.** The glottal jet uses the source law of §3 (obstacle form: it meets the false
folds), with a band up to 8 kHz and beyond. Because the law follows the instantaneous flow,
the noise is pulsed in voicing and steady in /h/ with no separate rule. Parameters: the two
constants of §3 at the glottis, the band. Its level in voicing is set by the measure of
problems.md §6 against a natural voice measured the same way (plan.md step 1).

## 5. Affricates: /tʃ dʒ/

A stop at the fricative's place, released slowly into that fricative: closure 40–70 ms, then
60–100 ms of frication for /tʃ/ (shorter and voiced for /dʒ/), with a rise time under 30–40
ms that tells it from /ʃ/ (Howell & Rosen 1983). Today the stop part seals for 27–36 ms or
leaks (0.003 cm² before /ɑ/), and the hill-climb log's experiment 15 found the frication cut
short by the folds' timing. Change: the closure target seals in every vowel context (aimed
past contact like the other stops); the release opens to the fricative's channel area at the
slow rate of a peeling blade; the glottal opening lasts to the end of the frication.
Parameters: closure share of the segment, release rate, the fricative's area.

## 6. Nasals: /m n ŋ/

**Physics.** The mouth is closed, the velum open: sound leaves by the nose. The murmur has a
strong first resonance at 250–300 Hz, weak higher ones, and a zero from the closed mouth
cavity at a frequency that falls as the cavity grows: 750–1250 Hz for /m/, 1450–2200 Hz for
/n/, above 3 kHz for /ŋ/ (Fujimura 1962). The murmur is 5–10 dB under the vowels beside it;
the abrupt change of spectrum at the closure and the release carries the place (Kurowski &
Blumstein 1984).

**Today.** Levels of −3…−13 dB, in range. The nasal tract is a 1D branch with one sinus; the
zeros were not measured. **Change.** None in the planner. The mouth cavity behind the
closure comes from the 3D geometry (its volume and length set the zero); the nose stays a 1D
branch in the real-time voice and in the precise render (design.md, deferred: no 3D nose).
The zeros are checked with the frequency-domain solver on the engine's airway.

## 7. Liquids and glides: /l ɹ w j/

**Physics and ranges.**

- /l/: the tip seals on the ridge, air passes one or both sides of the tongue. The pocket
  above the tongue behind the seal is a side branch and puts a zero near 2–3 kHz. Adult men:
  F1 about 340, F2 about 1180, F3 about 2520 Hz (Dalston 1975); a dark /ɫ/ has F2 near
  900–1000 Hz.
- /ɹ/: bunched or retroflex, with a space under the tongue and narrowed lips; its mark is a
  third formant under 2000 Hz. Adult men: F1 about 350, F2 about 1060, F3 about 1550 Hz
  (Dalston 1975; Espy-Wilson 1992; Delattre & Freeman 1968).
- /w/: F1 about 340, F2 about 730 Hz (Dalston 1975); /j/: F1 250–300, F2 above 2000 Hz.

**Today.** Levels in range. /l/'s side channels are an area scale on the midline tube, so the
pocket's zero cannot appear; /ɹ/'s sublingual space is part of the area law. **Change.** In
the 3D geometry these are shapes the tract already has (two channels at the /l/ cut; the
space under a raised tip). The real-time airway takes them as branches: the two side
channels as a parallel path round the seal with the pocket as its closed end, the sublingual
space as a side cavity (design.md §5.2). Acceptance is spectral (§9).

## 8. Where the 3D geometry enters

| Consonant property | From the midline today | From the 3D tract |
|---|---|---|
| Is the mouth sealed? | midline distance zero | the cut's area zero: a midline contact with open sides is /l/, a grooved contact is /s/ |
| Constriction area and shape | α·d^β with groove and lateral scales | the channel's own area, width and height in the cut |
| The jet's exit and what it hits | narrowest section; teeth by a distance window | the most anterior narrow cut; the first surface along the jet's axis and its distance |
| Front cavity | sections ahead of the constriction | the same plus the sublingual space, the vestibule between teeth and lips, the lips' horn |
| Side branches | one fixed piriform tube | piriform fossae, valleculae, the /l/ pocket, the sublingual space as branches |
| Wall area and tissue per section | shares by region | the cut's perimeter by tissue |

## 9. Acceptance lines

Measured as in problems.md §1 on the battery (stressed onsets /əˈCɑ/ and /ˈCɑ/ unless said)
and on the 20 training sentences, both voices. "Now" is the base, male / female.

| # | Line | Now | After |
|---|---|---|---|
| K1 | "can" in the operator's phrase: burst above 1.5 kHz re the vowel of "you" | −15.4 / −23.4 dB | ≥ −10 dB, both |
| K2 | "can": its vowel voiced | 0 % | ≥ 50 % of the planned span |
| K3 | "can": VOT | 68 ms | 25–45 ms |
| K4 | "can": loudest 5 ms in the 80 ms before the closure | −4.9 / −13.4 dB | ≤ −25 dB |
| S1 | /p t k b d g/ sealed (area ≤ 0.01 cm²), every context | 53–103 ms | ≥ 40 ms; affricates ≥ 30 ms in every context |
| S2 | mouth pressure at the release | voiceless 4.5–8.5; voiced −2.8…+9 cmH2O | voiceless 5–8; voiced 2–6, never negative |
| S3 | burst above 1.5 kHz re the vowel: /t/, /k/, /p/ | −11.0 / −20.6, −16.7 / −25.0, −21.9 / −32.7 dB | /t/ ≥ −14, /k/ ≥ −14, /p/ ≥ −22; none above −2; female within 4 dB of male |
| S4 | release noise within 10 dB of its peak, before voicing: /p/, /t/, /k/ | 8, 10, 17 ms | 5–15, 10–25, 15–35 ms, in that order |
| S5 | VOT, stressed onsets: /p/, /t/, /k/ | 63, 63, 65 ms | 45–70, 55–80, 60–90 ms, /p/ < /t/ < /k/; after /s/ 10–35 ms; /b d g/ 0–30 ms |
| S6 | a vowel after /p t k/, voiced share of its planned span (battery and sentences) | median 45 % stressed, 12–14 % unstressed | every one ≥ 60 % or ≥ 50 ms |
| S7 | aspiration re the vowel | −18…−20 / −28…−30 dB | −18…−8 dB, both |
| S8 | opening rate, first 5 ms | 54–120 (lips, tip), 19–78 (body) cm²/s | lips and tip 40–110, body 15–40 |
| S9 | loudest 5 ms in the 80 ms before any utterance-initial closure | median −26, worst −15 dB | ≤ −25 dB, every token |
| F1 | /s ʃ/ level re the vowel | −14…−17 / −25…−28 dB | −16…−6 dB, both; female within 3 dB of male |
| F2 | /f θ/ level | −23…−26 / −34…−37 dB | −28…−16 dB, and 8–15 dB under /s/ |
| F3 | spectral peak: /s/, /ʃ/ | 3.4 / 3.9 kHz, 2.3 / 2.5 kHz | /s/ 4–8 kHz, /ʃ/ 2.5–4 kHz, /s/ at least 1.4 × /ʃ/ |
| F4 | /z ʒ/: share of energy above 2.5 kHz, voicing kept (gate ≥ 0.8) | /z/ 0.47 / 0.16, /ʒ/ 0.03 / 0.01 | ≥ 0.25, and within 12 dB of /s ʃ/ above 2.5 kHz |
| F5 | frication within 10 dB of its peak | /f/ 15–21 ms of 100 | ≥ 60 % of the planned span, every voiceless fricative |
| F6 | mouth pressure, voiceless fricatives (lung pressure 8) | 2.3–4.4 cmH2O | 4.5–7.5 |
| H1 | /h/ level | −23 / −34 dB | −22…−10 dB, both |
| A1 | /tʃ/: frication at the level of /ʃ/ | about equal, cut short | within 3 dB of /ʃ/, ≥ 50 ms |
| N1 | nasal murmur level | −3…−13 dB | −14…−3 dB; zero in 750–1250 Hz for /m/, 1450–2200 Hz for /n/ in the airway's transfer function |
| L1 | /ɹ/ F3; /l/ F2 and F3; /w/ F2; /j/ F2 (male; female ×1.15–1.2) | not measured | /ɹ/ F3 ≤ 2000 Hz; /l/ F2 900–1400, F3 ≥ 2300; /w/ F2 ≤ 1000; /j/ F2 ≥ 2000 Hz |
| V1 | depth between a vowel's harmonics and their valleys at 4–8 kHz | 8.5 / 21.5 dB | within 4 dB of a natural voice of the same sex measured by the same script (plan.md step 1); if none can be measured, ≤ 8 dB both, stated as unreferenced |

v/ð are reported without a line: natural ones often have no frication.

## Sources

- Arkebauer, Hixon & Hardy (1967). Peak intraoral air pressures during speech. *J. Speech Hear. Res.* 10, 196–208.
- Behrens & Blumstein (1988). Acoustic characteristics of English voiceless fricatives. *J. Phonetics* 16, 295–298.
- Byrd (1993). 54,000 American stops. *UCLA Working Papers in Phonetics* 83, 97–116.
- Crystal & House (1988). Segmental durations in connected-speech signals. *J. Acoust. Soc. Am.* 83, 1553–1573.
- Dalston (1975). Acoustic characteristics of English /w, r, l/. *J. Acoust. Soc. Am.* 57, 462–469.
- Delattre & Freeman (1968). A dialect study of American r's by x-ray motion picture. *Linguistics* 44, 29–68.
- Espy-Wilson (1992). Acoustic measures for linguistic features distinguishing the semivowels /w j r l/. *J. Acoust. Soc. Am.* 92, 736–757.
- Fant (1960). *Acoustic Theory of Speech Production.* Mouton.
- Fujimura (1962). Analysis of nasal consonants. *J. Acoust. Soc. Am.* 34, 1865–1875.
- Hanson (1997). Glottal characteristics of female speakers. *J. Acoust. Soc. Am.* 101, 466–481.
- Hanson & Stevens (2000). Modeling stop-consonant releases for synthesis. ASA meeting poster, MIT RLE. †
- Hanson & Stevens (2003). Models of aspirated stops in English. *Proc. ICPhS*, 783–786. †
- Holmberg, Hillman & Perkell (1988). Glottal airflow and transglottal air pressure measurements. *J. Acoust. Soc. Am.* 84, 511–529.
- Howell & Rosen (1983). Production and perception of rise time in the voiceless affricate/fricative distinction. *J. Acoust. Soc. Am.* 73, 976–984.
- Jongman (1989). Duration of frication noise required for identification of English fricatives. *J. Acoust. Soc. Am.* 85, 1718–1725. †
- Jongman, Blumstein & Lahiri (1985). Acoustic properties for dental and alveolar stop consonants. *J. Phonetics* 13, 235–251.
- Jongman, Wayland & Wong (2000). Acoustic characteristics of English fricatives. *J. Acoust. Soc. Am.* 108, 1252–1263.
- Klatt (1975). Voice onset time, frication, and aspiration in word-initial consonant clusters. *J. Speech Hear. Res.* 18, 686–706.
- Klatt (1979). Synthesis by rule of segmental durations in English sentences. In *Frontiers of Speech Communication Research.*
- Klatt & Klatt (1990). Analysis, synthesis, and perception of voice quality variations. *J. Acoust. Soc. Am.* 87, 820–857.
- Klatt, Stevens & Mead (1968). Studies of articulatory activity and airflow during speech. *Ann. N.Y. Acad. Sci.* 155, 42–55.
- Kurowski & Blumstein (1984). Perceptual integration of the murmur and formant transitions for place of articulation in nasal consonants. *J. Acoust. Soc. Am.* 76, 383–390.
- Lisker & Abramson (1964). A cross-language study of voicing in initial stops. *Word* 20, 384–422.
- Lisker & Abramson (1967). Some effects of context on voice onset time in English stops. *Lang. Speech* 10, 1–28.
- Massey (1994). Transients at stop-consonant releases. S.M. thesis, MIT.
- Meyer-Eppler (1953). Zum Erzeugungsmechanismus der Geräuschlaute. *Z. Phonetik* 7, 196–212.
- Shadle (1985). The acoustics of fricative consonants. Ph.D. thesis, MIT (RLE Tech. Rep. 506).
- Shadle (1990). Articulatory-acoustic relationships in fricative consonants. In *Speech Production and Speech Modelling*, Kluwer.
- Stevens (1971). Airflow and turbulence noise for fricative and stop consonants: static considerations. *J. Acoust. Soc. Am.* 50, 1180–1192.
- Stevens (1998). *Acoustic Phonetics.* MIT Press.
- Subtelny, Worth & Sakuda (1966). Intraoral pressure and rate of flow during speech. *J. Speech Hear. Res.* 9, 498–518.
- Sundara (2005). Acoustic-phonetics of coronal stops: a cross-language study of Canadian English and Canadian French. *J. Acoust. Soc. Am.* 118, 1026–1037. †
- Zue (1976). Acoustic characteristics of stop consonants: a controlled study. Sc.D. thesis, MIT (Lincoln Lab. Tech. Rep. 523).
