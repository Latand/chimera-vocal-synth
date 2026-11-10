/**
 * The 3D head and vocal tract every screen draws: the lab, the face screen of each mode, the
 * duet page. One scene, three.js.
 *
 *  - the tract is the engine's 3D model (`chimera::anatomy::tract3d`): its element meshes for
 *    the posture, shaded by material (tongue, mucosa, enamel, gum, …);
 *  - round it, the head's skin (asset.ts, rig.ts), following the engine's jaw and lips;
 *  - opened (`cutaway`), the half of the head towards the viewer is hidden and the cut face
 *    shows its midsagittal section (section.ts), with the tract's air as a hole in it.
 * Nothing here feeds the sound, and no shape is computed here: positions come from the engine
 * and from the skin's rig.
 */
import * as THREE from "three";
import { RoomEnvironment } from "three/addons/environments/RoomEnvironment.js";
import type { HumanArtic, Tract3dShape, Tract3dView } from "../../../wasm/types.ts";
import type { VoiceModel } from "../../analysis.ts";
import { bodyWeights } from "../../body.ts";
import { V } from "../../colors.ts";
import type { HeadMesh } from "./asset.ts";
import { HeadRig } from "./rig.ts";
import { SectionPainter, sectionBounds } from "./section.ts";

export interface SceneOptions {
  /** the head's skin round the tract */
  head: boolean;
  /** hide the half towards the viewer and show the cut face */
  cutaway: boolean;
  /** the skin translucent (with the head whole) */
  seeThrough: boolean;
  /** elements not drawn */
  hidden: readonly string[];
  /** the element whose outline grid is drawn (lab) */
  lit: string | null;
  /** pull the elements apart (tract alone) */
  explode: boolean;
  /** which half an opened head keeps: 1 the speaker's left (seen from their right), −1 the other */
  side?: 1 | -1;
}

export type ViewName = "side" | "front" | "three" | "top" | "mouth";

/** the elements' colours by material */
const MATERIALS: Record<string, { color: number; roughness: number; coat: number }> = {
  tongue: { color: 0xc4696f, roughness: 0.5, coat: 0.35 },
  floor: { color: 0xb8505a, roughness: 0.45, coat: 0.4 },
  palate: { color: 0xe0a39a, roughness: 0.5, coat: 0.3 },
  velum: { color: 0xd47f86, roughness: 0.45, coat: 0.4 },
  pharynx: { color: 0xb5525a, roughness: 0.5, coat: 0.35 },
  larynx: { color: 0xcf8f86, roughness: 0.5, coat: 0.3 },
  epiglottis: { color: 0xe7c2ac, roughness: 0.5, coat: 0.25 },
  lip: { color: 0xc96a70, roughness: 0.45, coat: 0.35 },
  enamel: { color: 0xf4efe3, roughness: 0.22, coat: 0.6 },
  gum: { color: 0xdc8d92, roughness: 0.4, coat: 0.5 },
  cheek: { color: 0xc0606a, roughness: 0.48, coat: 0.35 },
};

/** linear RGB of an sRGB hex */
const lin = (hex: number): [number, number, number] => {
  const c = new THREE.Color(hex);
  return [c.r, c.g, c.b];
};
const SKIN = lin(0xd9b09c), LIP = lin(0xb9776f), BLUSH = lin(0xd29382), SHADE = lin(0x9c6a5c), STAND = lin(0xcfa792);

interface Part {
  element: string;
  group: THREE.Group;
  geo: THREE.BufferGeometry;
  mesh: THREE.Mesh;
  lines: THREE.LineSegments;
  position: THREE.BufferAttribute | null;
}

export class HeadScene {
  readonly scene = new THREE.Scene();
  readonly camera = new THREE.PerspectiveCamera(30, 1, 0.5, 600);
  private readonly clip = new THREE.Plane(new THREE.Vector3(0, 0, -1), 0);
  private readonly materials = new Map<string, THREE.MeshPhysicalMaterial>();
  private readonly parts: Part[] = [];
  private readonly env: THREE.Texture;
  private readonly key: THREE.DirectionalLight;
  private model: VoiceModel | null = null;
  private headMesh: HeadMesh | null = null;
  private rig: HeadRig | null = null;
  private readonly headGeo = new THREE.BufferGeometry();
  private readonly skinMat: THREE.MeshPhysicalMaterial;
  private readonly insideMat: THREE.MeshStandardMaterial;
  private readonly skin: THREE.Mesh;
  private readonly inside: THREE.Mesh;
  private painter: SectionPainter | null = null;
  private sectionTex: THREE.CanvasTexture | null = null;
  private readonly sectionMat: THREE.MeshStandardMaterial;
  private readonly section: THREE.Mesh;
  private readonly centre: THREE.Line;
  private readonly cutAir: THREE.LineSegments;
  private readonly cutFrame: THREE.LineLoop;
  /** the last view computed (for overlays that read the lips or the jaw) */
  view: Tract3dView | null = null;

  constructor(renderer: THREE.WebGLRenderer) {
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    // a tone map that keeps the skin's own colour in the lights (the filmic ones bleach it)
    renderer.toneMapping = THREE.NeutralToneMapping;
    renderer.toneMappingExposure = 1.0;
    renderer.localClippingEnabled = true;
    // the key light casts shadows: the nose's on the cheek, the chin's on the neck, and the
    // mouth is dark inside wherever the light does not reach through the lips
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFShadowMap;
    // light: a soft room all round (so no side is black), a warm key that travels with the
    // camera from above its shoulder, a cool rim from behind
    const pmrem = new THREE.PMREMGenerator(renderer);
    this.env = pmrem.fromScene(new RoomEnvironment(), 0.04).texture;
    pmrem.dispose();
    this.scene.environment = this.env;
    this.scene.environmentIntensity = 0.42;
    const key = new THREE.DirectionalLight(0xfff3e8, 2.0);
    key.position.set(-22, 24, 30);
    key.castShadow = true;
    key.shadow.mapSize.set(2048, 2048);
    key.shadow.bias = -0.001;
    key.shadow.normalBias = 0.12;
    key.shadow.radius = 5;
    this.key = key;
    this.camera.add(key);
    this.scene.add(key.target);
    const fill = new THREE.DirectionalLight(0xdfe8ff, 0.22);
    fill.position.set(26, -4, 14);
    this.camera.add(fill);
    this.scene.add(this.camera);
    const rim = new THREE.DirectionalLight(0xb9d2ff, 0.7);
    rim.position.set(-30, 30, -45);
    this.scene.add(rim);

    for (const [name, m] of Object.entries(MATERIALS)) {
      this.materials.set(
        name,
        new THREE.MeshPhysicalMaterial({ color: m.color, roughness: m.roughness, metalness: 0, clearcoat: m.coat, clearcoatRoughness: 0.3, side: THREE.DoubleSide, envMapIntensity: 0.9, clipShadows: true, shadowSide: THREE.DoubleSide }),
      );
    }

    // skin: colours per vertex; light that enters the skin comes back out red, so the edge of
    // every shadow is warm (a wrapped, tinted diffuse term) and the surface never looks like plastic
    this.skinMat = new THREE.MeshPhysicalMaterial({ vertexColors: true, roughness: 0.56, metalness: 0, sheen: 0.18, sheenRoughness: 0.55, sheenColor: new THREE.Color(0xffc4b0), envMapIntensity: 0.8, clipShadows: true, shadowSide: THREE.DoubleSide });
    this.skinMat.onBeforeCompile = (shader) => {
      const chunk = THREE.ShaderChunk.lights_physical_pars_fragment.replace(
        "vec3 irradiance = dotNL * directLight.color;",
        `float skinNL = dot( geometryNormal, directLight.direction );
	float skinWrap = saturate( ( skinNL + 0.32 ) / 1.32 );
	vec3 irradiance = mix( skinWrap * vec3( 0.95, 0.42, 0.32 ) * 0.5, vec3( dotNL ), smoothstep( 0.0, 0.5, skinNL ) ) * directLight.color;`,
      );
      shader.fragmentShader = shader.fragmentShader.replace("#include <lights_physical_pars_fragment>", chunk);
    };
    this.insideMat = new THREE.MeshStandardMaterial({ color: 0x9c4a4f, roughness: 0.6, metalness: 0, side: THREE.BackSide, envMapIntensity: 0.3 });
    this.skin = new THREE.Mesh(this.headGeo, this.skinMat);
    this.inside = new THREE.Mesh(this.headGeo, this.insideMat);
    this.skin.castShadow = this.skin.receiveShadow = this.inside.receiveShadow = true;
    this.skin.visible = this.inside.visible = false;
    this.scene.add(this.skin, this.inside);

    this.sectionMat = new THREE.MeshStandardMaterial({ roughness: 0.78, metalness: 0, alphaTest: 0.5, side: THREE.DoubleSide, envMapIntensity: 0.5 });
    this.section = new THREE.Mesh(new THREE.PlaneGeometry(1, 1), this.sectionMat);
    this.section.visible = false;
    this.scene.add(this.section);

    const flat = (color: string, opacity = 1) => new THREE.LineBasicMaterial({ color: new THREE.Color(color), transparent: opacity < 1, opacity, depthTest: false, toneMapped: false });
    this.centre = new THREE.Line(new THREE.BufferGeometry(), new THREE.LineDashedMaterial({ color: new THREE.Color(V.pink), dashSize: 0.25, gapSize: 0.18, transparent: true, opacity: 0.85, depthTest: false, toneMapped: false }));
    this.cutAir = new THREE.LineSegments(new THREE.BufferGeometry(), flat(V.amber));
    this.cutFrame = new THREE.LineLoop(new THREE.BufferGeometry(), flat(V.amber, 0.5));
    for (const o of [this.centre, this.cutAir, this.cutFrame]) {
      o.renderOrder = 3000;
      o.visible = false;
      this.scene.add(o);
    }
  }

  /** The head's skin, once it has loaded. */
  setHead(mesh: HeadMesh): void {
    this.headMesh = mesh;
    this.rig = null;
  }

  private buildRig(model: VoiceModel): void {
    const mesh = this.headMesh!;
    const w = bodyWeights(model.body);
    const of = (name: string) => w.find(([n]) => n === name)?.[1] ?? 0;
    const rig = new HeadRig(mesh, { weights: [of("male"), of("female"), of("child")], k: model.bodyParams.scale, ph: model.bodyParams.pharynx });
    this.rig = rig;
    const k = model.bodyParams.scale;
    const n = mesh.n;
    // colours: the lips' vermilion, warmth over the cheeks, the nose, the ears and the chin, a
    // little shade in the eye sockets and in every crease (the nostrils, the ear's bowl, the
    // lids' line, the corners of the mouth); the stand in one tone
    const colors = new Float32Array(3 * n);
    const hollow = hollows(rig.rest, mesh.indices, n, k);
    const g = (a: number, b: number, c = 0) => Math.exp(-(a * a + b * b + c * c));
    for (let i = 0; i < n; i++) {
      const x = rig.rest[3 * i] / k, y = rig.rest[3 * i + 1] / k, z = Math.abs(rig.rest[3 * i + 2]) / k;
      const t = Math.min(1, Math.max(0, (mesh.tint[i] - 0.3) / 0.5));
      const lip = t * t * (3 - 2 * t);
      const blush = Math.min(1, 0.5 * g((y - 11.6) / 1.7, (z - 4.3) / 1.5, (x - 4.5) / 2.6) + 0.4 * g((y - 11.2) / 1.0, z / 1.0, (x - 7.6) / 1.2) + 0.45 * g((y - 13.2) / 2.4, (z - 7.4) / 0.9, (x + 2.2) / 2.0) + 0.2 * g((y - 4.9) / 1.0, z / 1.4, (x - 7.0) / 1.2));
      const shade = Math.min(0.85, 0.35 * g((y - 15.4) / 0.9, (z - 3.1) / 1.3, (x - 6.2) / 1.6) + hollow[i]);
      for (let c = 0; c < 3; c++) {
        let v = SKIN[c] * (1 - blush) + BLUSH[c] * blush;
        v = v * (1 - lip) + LIP[c] * lip;
        v = v * (1 - shade) + SHADE[c] * v * 1.6 * shade;
        colors[3 * i + c] = mesh.stand[i] ? STAND[c] : v;
      }
    }
    this.headGeo.setAttribute("position", new THREE.BufferAttribute(rig.out, 3).setUsage(THREE.DynamicDrawUsage));
    this.headGeo.setAttribute("color", new THREE.BufferAttribute(colors, 3));
    this.headGeo.setIndex(new THREE.BufferAttribute(mesh.indices, 1));
    const b = sectionBounds(k);
    this.painter = new SectionPainter(b);
    this.sectionTex?.dispose();
    this.sectionTex = new THREE.CanvasTexture(this.painter.canvas);
    this.sectionTex.colorSpace = THREE.SRGBColorSpace;
    this.sectionTex.anisotropy = 4;
    this.sectionMat.map = this.sectionTex;
    this.sectionMat.needsUpdate = true;
    this.section.scale.set(b.x1 - b.x0, b.y1 - b.y0, 1);
    this.section.position.set((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2, 0);
  }

  /**
   * Draw `model` in posture `a` (with the lab's 3D offsets `shape`). Returns the engine's view
   * of the posture.
   */
  update(model: VoiceModel, a: HumanArtic, shape: Tract3dShape, o: SceneOptions): Tract3dView {
    const fresh = model !== this.model;
    if (fresh) {
      this.model = model;
      this.rig = null;
      for (const p of this.parts) {
        this.scene.remove(p.group);
        p.geo.dispose();
        (p.lines.material as THREE.Material).dispose();
      }
      this.parts.length = 0;
    }
    const view = model.view(a, shape, this.parts.length === 0);
    this.view = view;
    const side = o.side ?? 1;
    this.clip.normal.set(0, 0, -side);
    const planes = o.cutaway ? [this.clip] : [];
    const k = model.bodyParams.scale;
    const withHead = o.head && !!this.headMesh;

    // ---- the tract's elements
    if (this.parts.length === 0) {
      for (const m of view.meshes) {
        const geo = new THREE.BufferGeometry();
        const position = new THREE.BufferAttribute(new Float32Array(3 * m.count), 3).setUsage(THREE.DynamicDrawUsage);
        geo.setAttribute("position", position);
        geo.setIndex(m.indices!);
        const mats = (m.groups ?? []).map((g, i) => {
          geo.addGroup(g.start, g.count, i);
          return this.materials.get(g.material) ?? this.materials.get("cheek")!;
        });
        const mesh = new THREE.Mesh(geo, mats);
        mesh.castShadow = mesh.receiveShadow = true;
        const lineGeo = new THREE.BufferGeometry();
        lineGeo.setAttribute("position", position);
        lineGeo.setIndex(m.lines!);
        const lines = new THREE.LineSegments(lineGeo, new THREE.LineBasicMaterial({ color: 0xffffff, transparent: true, opacity: 0.5, toneMapped: false }));
        lines.renderOrder = 900;
        const group = new THREE.Group();
        group.add(mesh, lines);
        this.scene.add(group);
        this.parts.push({ element: m.element, group, geo, mesh, lines, position });
      }
    }
    view.meshes.forEach((m, i) => {
      const p = this.parts[i];
      (p.position!.array as Float32Array).set(view.positions.subarray(3 * m.start, 3 * (m.start + m.count)));
      p.position!.needsUpdate = true;
      p.geo.computeVertexNormals();
      p.geo.computeBoundingSphere();
      // inside a whole head the lips' and cheeks' inner surfaces lie under the skin's own
      const covered = withHead && !o.cutaway && (m.element === "lips" || m.element === "cheeks");
      p.group.visible = !o.hidden.includes(m.element) && !covered;
      p.lines.visible = o.lit === m.element;
      (p.lines.material as THREE.LineBasicMaterial).clippingPlanes = planes;
      const n = view.meshes.length;
      p.group.position.set(0, o.explode && !withHead ? (i - (n - 1) / 2) * 0.9 * k : 0, o.explode && !withHead ? (i % 2 ? 1 : -1) * 1.2 * k : 0);
    });
    // inside a whole head the room's light does not reach the mouth: what lights it is the key,
    // through the lips
    const closed = withHead && !o.cutaway && !o.seeThrough;
    for (const m of this.materials.values()) {
      m.clippingPlanes = planes;
      m.envMapIntensity = closed ? 0.12 : 0.9;
    }
    // the key's shadow frame: the head (or the tract alone), wherever the camera carries the light
    const reach = (withHead ? 21 : 9) * k;
    this.key.target.position.set((withHead ? -3.5 : 3) * k, (withHead ? 8 : 5.5) * k, 0);
    const sc = this.key.shadow.camera;
    if (sc.right !== reach) {
      sc.left = sc.bottom = -reach;
      sc.right = sc.top = reach;
      sc.near = 1;
      sc.far = 600;
      sc.updateProjectionMatrix();
    }

    // ---- the head
    if (withHead && !this.rig) this.buildRig(model);
    this.skin.visible = this.inside.visible = withHead;
    this.section.visible = withHead && o.cutaway;
    if (withHead && this.rig) {
      const rig = this.rig, mesh = this.headMesh!;
      rig.pose(view.display, a.lipWidth);
      this.headGeo.attributes.position.needsUpdate = true;
      this.headGeo.computeVertexNormals();
      this.headGeo.computeBoundingSphere();
      const see = o.seeThrough && !o.cutaway;
      this.skinMat.clippingPlanes = this.insideMat.clippingPlanes = planes;
      // (a material compiled opaque keeps its alpha at 1 until it is compiled again)
      if (this.skinMat.transparent !== see) this.skinMat.needsUpdate = true;
      this.skinMat.transparent = see;
      this.skinMat.opacity = see ? 0.3 : 1;
      this.skinMat.depthWrite = !see;
      this.skin.castShadow = !see;
      this.skin.renderOrder = see ? 2000 : 0;
      this.inside.visible = !see;
      if (o.cutaway && this.painter) {
        const pts = (ids: Int32Array) => Array.from(ids, (i) => ({ x: rig.out[3 * i], y: rig.out[3 * i + 1] }));
        const count = (ids: Int32Array, from: number, step: number) => {
          let c = 0;
          for (let j = from; j >= 0 && j < ids.length && mesh.stand[ids[j]]; j += step) c++;
          return c;
        };
        this.painter.paint({
          model,
          display: view.display,
          front: pts(mesh.midFront),
          back: pts(mesh.midBack),
          standFront: count(mesh.midFront, 0, 1),
          standBack: count(mesh.midBack, mesh.midBack.length - 1, -1),
          larynx: a.larynx,
        });
        this.sectionTex!.needsUpdate = true;
        this.section.position.z = 0.004 * side;
      }
    }
    return view;
  }

  /** The lab's drawings in the scene: the centreline, and a cut's air with its frame. */
  overlay(o: { centreline?: { x: number; y: number }[]; segments?: number[]; frame?: number[] } | null): void {
    const set = (obj: THREE.Line, pts: number[] | undefined) => {
      obj.visible = !!pts && pts.length > 0;
      if (pts) obj.geometry.setAttribute("position", new THREE.Float32BufferAttribute(pts, 3));
    };
    set(this.centre, o?.centreline?.flatMap((p) => [p.x, p.y, 0]));
    if (this.centre.visible) this.centre.computeLineDistances();
    set(this.cutAir, o?.segments);
    set(this.cutFrame, o?.frame);
  }

  /**
   * Where the camera stands for a named view: its position and what it looks at. The whole head
   * (or the tract alone) fits the camera's frame from there, with `margin` to spare.
   */
  pose(name: ViewName, withHead: boolean, side: 1 | -1 = 1, margin = 1.1, vFill = 1): { position: THREE.Vector3; target: THREE.Vector3 } {
    const k = this.model?.bodyParams.scale ?? 1;
    const aspect = this.camera.aspect || 1;
    const half = Math.tan((this.camera.fov * Math.PI) / 360);
    if (name === "mouth") {
      const target = new THREE.Vector3(6.4 * k, 7.5 * k, 0);
      const dir = new THREE.Vector3(1, 0.05, 0.36 * side).normalize();
      return { target, position: target.clone().addScaledVector(dir, (5.2 * k * margin) / (half * Math.min(1, aspect))) };
    }
    const dirs: Record<Exclude<ViewName, "mouth">, [number, number, number]> = { side: [0, 0.02, 1], front: [1, 0.04, 0.001], top: [0.05, 1, 0.02], three: [0.78, 0.2, 0.62] };
    const d = dirs[name];
    const dir = new THREE.Vector3(d[0], d[1], d[2] * side).normalize();
    // the box of what is shown (template cm): the head on its stand, or the tract
    const [x0, x1, y0, y1, z1] = withHead ? [-15.4, 8.4, -10.8, 24.8, 9] : [-2.2, 8.2, -1.2, 12.2, 3.6];
    const target = new THREE.Vector3(((x0 + x1) / 2) * k, ((y0 + y1) / 2) * k, 0);
    // its half-extents across and up the screen from this direction
    const right = new THREE.Vector3().crossVectors(new THREE.Vector3(0, 1, 0), dir).normalize();
    const up = new THREE.Vector3().crossVectors(dir, right);
    let hw = 0, hh = 0, deep = 0;
    for (const cx of [x0, x1]) for (const cy of [y0, y1]) for (const cz of [-z1, z1]) {
      const v = new THREE.Vector3(cx * k, cy * k, cz * k).sub(target);
      hw = Math.max(hw, Math.abs(v.dot(right)));
      hh = Math.max(hh, Math.abs(v.dot(up)));
      deep = Math.max(deep, v.dot(dir));
    }
    // (the box's corners stand outside the head's own rounded outline: from an oblique direction
    // a tenth of the fit is air)
    const round = name === "three" || name === "top" ? 0.86 : 0.96;
    // (`vFill`: the share of the view's height the head may fill, when rows of text take the rest)
    const dist = deep + margin * round * Math.max(hh / (half * vFill), hw / (half * aspect));
    return { target, position: target.clone().addScaledVector(dir, dist) };
  }

  /** a body point on the canvas (CSS px), for labels drawn over the 3D view */
  project(x: number, y: number, z: number, w: number, h: number): [number, number] {
    const p = new THREE.Vector3(x, y, z).project(this.camera);
    return [(p.x * 0.5 + 0.5) * w, (0.5 - p.y * 0.5) * h];
  }

  dispose(): void {
    for (const p of this.parts) {
      p.geo.dispose();
      p.lines.geometry.dispose();
      (p.lines.material as THREE.Material).dispose();
    }
    for (const m of this.materials.values()) m.dispose();
    this.headGeo.dispose();
    this.skinMat.dispose();
    this.insideMat.dispose();
    this.section.geometry.dispose();
    this.sectionMat.dispose();
    this.sectionTex?.dispose();
    this.env.dispose();
    this.key.shadow.dispose();
    for (const o of [this.centre, this.cutAir, this.cutFrame]) {
      o.geometry.dispose();
      (o.material as THREE.Material).dispose();
    }
  }
}

/**
 * How deep in a crease each vertex of the skin lies, 0…0.6: the surface's mean curvature at
 * rest (its neighbours' centre above the vertex, along the normal), smoothed. A crease keeps
 * less of the room's light than open skin, and no light in the scene knows that.
 */
function hollows(rest: Float32Array, indices: Uint32Array, n: number, k: number): Float32Array {
  const sum = new Float32Array(3 * n), count = new Uint16Array(n), normal = new Float32Array(3 * n), edge2 = new Float32Array(n);
  for (let t = 0; t < indices.length; t += 3) {
    const a = indices[t], b = indices[t + 1], c = indices[t + 2];
    const ax = rest[3 * a], ay = rest[3 * a + 1], az = rest[3 * a + 2];
    const ux = rest[3 * b] - ax, uy = rest[3 * b + 1] - ay, uz = rest[3 * b + 2] - az;
    const vx = rest[3 * c] - ax, vy = rest[3 * c + 1] - ay, vz = rest[3 * c + 2] - az;
    const nx = uy * vz - uz * vy, ny = uz * vx - ux * vz, nz = ux * vy - uy * vx;
    for (const [i, j] of [[a, b], [b, c], [c, a]] as const) {
      normal[3 * i] += nx;
      normal[3 * i + 1] += ny;
      normal[3 * i + 2] += nz;
      // (an edge is seen once from each of its two triangles: each vertex gets its neighbour twice, evenly)
      let l2 = 0;
      for (let q = 0; q < 3; q++) {
        sum[3 * i + q] += rest[3 * j + q];
        sum[3 * j + q] += rest[3 * i + q];
        l2 += (rest[3 * j + q] - rest[3 * i + q]) ** 2;
      }
      edge2[i] += l2;
      edge2[j] += l2;
      count[i]++;
      count[j]++;
    }
  }
  let h = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    if (count[i] === 0) continue;
    const nl = Math.hypot(normal[3 * i], normal[3 * i + 1], normal[3 * i + 2]) || 1;
    let d = 0;
    for (let q = 0; q < 3; q++) d += (sum[3 * i + q] / count[i] - rest[3 * i + q]) * (normal[3 * i + q] / nl);
    // the curvature (1/cm, in the template's size): the neighbours' height over the vertex by
    // their mean squared distance. A fold tighter than 1 cm across is a full crease; the neck's
    // and the temples' gentle hollows (several cm) are none
    const curve = (4 * d * k) / Math.max(1e-6, edge2[i] / count[i]);
    h[i] = Math.min(1, Math.max(0, (curve - 0.35) / 1.4));
  }
  // spread over the neighbours twice, so a crease is a soft band and not a line of dark vertices
  for (let pass = 0; pass < 2; pass++) {
    const acc = new Float32Array(n);
    for (let t = 0; t < indices.length; t += 3) {
      const a = indices[t], b = indices[t + 1], c = indices[t + 2];
      acc[a] += h[b] + h[c];
      acc[b] += h[c] + h[a];
      acc[c] += h[a] + h[b];
    }
    const next = new Float32Array(n);
    for (let i = 0; i < n; i++) next[i] = count[i] ? 0.5 * h[i] + 0.5 * (acc[i] / count[i]) : 0;
    h = next;
  }
  for (let i = 0; i < n; i++) h[i] = 0.6 * Math.min(1, h[i]);
  return h;
}
