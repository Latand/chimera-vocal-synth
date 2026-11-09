import { useStore } from "../store.ts";
import { creatureById } from "../../wasm/engine.ts";
import { playDemo, DEMO_SONGS } from "../singing.ts";

/** The prompt over a mode's main screen until the audio runs: what the mode is, and one button that starts it sounding. */
export function StartPrompt() {
  const started = useStore((s) => s.started);
  const starting = useStore((s) => s.starting);
  const err = useStore((s) => s.startError);
  const start = useStore((s) => s.start);
  const setHold = useStore((s) => s.setHold);
  const mode = useStore((s) => s.mode);
  const playCall = useStore((s) => s.playCall);
  const creature = useStore((s) => s.creature);
  const setBeatPlaying = useStore((s) => s.setBeatPlaying);
  const threeD = useStore((s) => s.view.threeD);
  if (started) return null;
  const copy = {
    bestiary: ["Each animal's sound comes from a simulation of its own sound source and airway.", "Start and hear a call", "Then play its calls, or hold the keys to sustain it at a pitch."],
    beatbox: ["Every drum here is a mouth sound: lips, tongue and larynx pushing, sealing and sucking air.", "Start the beat", "Then edit the pattern, tap the pads, or hum a bass note on the keys."],
    speak: ["Every sound here is produced by simulating air moving through a throat, mouth and nose.", "Start the voice", "Then type something and press Speak, or tap a sound in the chart."],
    sing: ["Every sound here is produced by simulating air moving through a throat, mouth and nose.", "Start and hear a song", "Then play the keys: each note sings the next syllable of the lyrics."],
    lab: ["The head and its vocal tract in 3D. The posture you set is the posture you hear.", "Start and listen", "Then pick a posture, turn a control, or type a word and press Say."],
  }[mode as "bestiary" | "beatbox" | "speak" | "sing" | "lab"] ?? [
    "Every sound here is produced by simulating air moving through a throat, mouth and nose.",
    "Breathe in and start",
    threeD ? "Then play the keys or the vowel map. Turn 3D off to drag the tongue, lips and jaw." : "Then drag the tongue, lips or jaw, or play the keys.",
  ];
  return (
    <div className="start-prompt" data-layout="overlay">
      <p>{copy[0]}</p>
      <button
        className="primary large"
        disabled={starting}
        onClick={async () => {
          // the demo waits for audio itself, so a stop meanwhile (body, mode) cancels it
          if (mode === "sing") return void playDemo(DEMO_SONGS[0].name, useStore.getState().body, start);
          await start();
          const first = creatureById(creature)?.calls[0];
          if (mode === "bestiary" && first) playCall(first.id);
          else if (mode === "beatbox") setBeatPlaying(true);
          else if ((mode === "play" || mode === "lab") && useStore.getState().mode === mode) setHold(true);
        }}
      >
        {starting ? "Starting…" : copy[1]}
      </button>
      {err ? <p className="error">Audio could not start: {err}. Reload the page and try again.</p> : <p className="hint">{copy[2]}</p>}
    </div>
  );
}
