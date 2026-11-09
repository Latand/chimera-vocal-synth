/**
 * Lab mode's vocabulary: the 3D model's elements (names, colours, what they are), the midline
 * articulators' pages, and readable names for parameters. The geometry itself is the engine's
 * (`chimera::anatomy::tract3d`); nothing here computes shape.
 */
import { V, mix } from "./colors.ts";
import type { ArticKey } from "../wasm/engine.ts";

export interface LabElement {
  key: string;
  label: string;
  color: string;
  about: string;
}

/** in the order the engine returns the meshes */
export const ELEMENTS: LabElement[] = [
  { key: "tongue", label: "Tongue", color: V.pink, about: "Surface over five stations, root to tip: margins raised or lowered, a midline groove, width, tip shape." },
  { key: "palate", label: "Palate", color: V.blue, about: "Hard palate: the vault falls from the midline to the gum line at the dental arch." },
  { key: "velum", label: "Velum & uvula", color: mix(V.blue, V.white, 0.45), about: "Soft palate with the uvula, the small lobe hanging from the middle of its free edge; beside it the edge rises into the faucial arches." },
  { key: "pharynx", label: "Pharynx", color: mix(V.blue, V.green, 0.35), about: "Posterior and lateral walls: a half-width per level and the rounding of the corners." },
  { key: "larynx", label: "Larynx", color: V.green, about: "Laryngeal tube (the engine's epilarynx profile), epiglottis, piriform fossae, valleculae." },
  { key: "lips", label: "Lips", color: V.red, about: "Inner surfaces of the lips; the opening narrows to the mouth's corners." },
  { key: "teethJaw", label: "Teeth & jaw", color: V.white, about: "Upper and lower dental arches (the lower carried by the jaw) and the mandible." },
  { key: "cheeks", label: "Cheeks", color: mix(V.pink, V.blue, 0.55), about: "The cheeks' inner surface beside the teeth, with the vestibule between." },
];
export const MIDLINE = "midline";

/** the 17 articulators in pages of four, by what they move */
export const MIDLINE_PAGES: { name: string; keys: ArticKey[] }[] = [
  { name: "Tongue body", keys: ["tbX", "tbY", "root", "dorsum"] },
  { name: "Tip", keys: ["ttX", "ttY", "ttCurl", "lateral"] },
  { name: "Jaw & lips", keys: ["jaw", "lipAp", "lipProt", "lipWidth"] },
  { name: "Throat", keys: ["velum", "larynx", "epilarynx", "ventricular"] },
  { name: "Labiodental", keys: ["labiodental"] },
];

const NAMES: Record<string, string> = {
  tbX: "Body front",
  tbY: "Body height",
  root: "Root",
  dorsum: "Dorsum hollow",
  ttX: "Tip front",
  ttY: "Tip height",
  ttCurl: "Tip curl",
  lateral: "Lateral",
  jaw: "Jaw open",
  lipAp: "Lip gap",
  lipProt: "Protrude",
  lipWidth: "Lip spread",
  velum: "Velum port",
  larynx: "Larynx",
  epilarynx: "Twang",
  ventricular: "False folds",
  labiodental: "Lip to teeth",
};
/** short label for an articulator or a 3D parameter key */
export function paramLabel(key: string): string {
  return NAMES[key] ?? key.replace(/([A-Z])/g, " $1").replace(/^./, (x) => x.toUpperCase());
}

/** pages of the 3D parameters of one element, grouped by name where it helps */
export function shapePages<T extends { key: string; element: string }>(specs: T[], element: string): { name: string; specs: T[] }[] {
  const own = specs.filter((s) => s.element === element);
  if (element === "tongue") {
    const by = (pred: (k: string) => boolean) => own.filter((s) => pred(s.key));
    return [
      { name: "Margins", specs: by((k) => k.startsWith("side") && k !== "sideAsym").slice(0, 4) },
      { name: "Margins 2", specs: [...by((k) => k === "sideTip" || k === "sideAsym")] },
      { name: "Groove", specs: by((k) => k.startsWith("groove") && !["grooveWidth", "grooveTaper"].includes(k)).slice(0, 4) },
      { name: "Groove 2", specs: by((k) => ["grooveTip", "grooveWidth", "grooveTaper"].includes(k)) },
      { name: "Size", specs: by((k) => ["tongueWidth", "tipWidth", "edgeRound"].includes(k)) },
    ].filter((p) => p.specs.length > 0);
  }
  const pages: { name: string; specs: T[] }[] = [];
  for (let i = 0; i < own.length; i += 4) pages.push({ name: own.length > 4 ? `Page ${i / 4 + 1}` : "All", specs: own.slice(i, i + 4) });
  return pages;
}

/** value in use for a 3D parameter → the offset the engine takes (offsets are from `derive`) */
export function offsetFor(value: number, derived: number, fallback: number): number {
  return value - derived + fallback;
}
