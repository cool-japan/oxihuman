/*
 * OxiHuman BodyLab — src/viewer.js
 * Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
 * SPDX-License-Identifier: Apache-2.0
 *
 * three.js (WebGL2) viewer. Builds a BufferGeometry directly over the engine's
 * zero-copy WASM memory views (positions / normals / uvs / indices), re-views on
 * generation or buffer-identity change, and updates in place via
 * attribute.needsUpdate when only the vertex data changed. Soft studio lighting,
 * a matte skin-adjacent MeshPhysicalMaterial, a shadow-catcher floor, gentle
 * turntable OrbitControls, and a render-level idle breath that never touches
 * engine parameters.
 */

import * as THREE from 'three';
import { OrbitControls } from '../vendor/OrbitControls.js';

export class Viewer {
  /** @param {HTMLCanvasElement} canvas */
  constructor(canvas) {
    this.canvas = canvas;
    this.engine = null;
    this.memory = null;

    this.geometry = null;
    this.mesh = null;
    this.gen = -1;
    this.pendingMorph = false;
    this.figureHeight = 1;

    // zero-copy views (created in _makeViews)
    this.positions = new Float32Array(0);
    this.normals = new Float32Array(0);
    this.uvs = new Float32Array(0);
    this.indices = new Uint32Array(0);

    // FPS rolling window
    this._frameTimes = [];
    this._lastNow = 0;

    this._initRenderer();
    this._initScene();
    this._initLights();
    this._initFloor();
    this._initControls();
    this._observeResize();
  }

  _initRenderer() {
    // alpha:true lets the CSS studio gradient show behind the shadow catcher.
    this.renderer = new THREE.WebGLRenderer({
      canvas: this.canvas,
      antialias: true,
      alpha: true,
      powerPreference: 'high-performance',
    });
    this.renderer.setClearColor(0x000000, 0);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.05;
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  }

  _initScene() {
    this.scene = new THREE.Scene();
    this.camera = new THREE.PerspectiveCamera(36, 1, 0.01, 100000);
    this.camera.position.set(0, 12, 46);
  }

  _initLights() {
    // Soft studio rig: key + fill + rim + hemisphere ambient.
    const key = new THREE.DirectionalLight(0xffffff, 2.6);
    key.position.set(3, 8, 6);
    key.castShadow = true;
    key.shadow.mapSize.set(2048, 2048);
    key.shadow.bias = -0.0006;
    key.shadow.normalBias = 0.02;
    this.keyLight = key;
    this.scene.add(key);
    this.scene.add(key.target);

    const fill = new THREE.DirectionalLight(0xb9c7ff, 0.9);
    fill.position.set(-6, 4, 4);
    this.scene.add(fill);

    const rim = new THREE.DirectionalLight(0xd9c6ff, 1.7);
    rim.position.set(-2, 6, -8);
    this.scene.add(rim);

    const hemi = new THREE.HemisphereLight(0xdfe6ff, 0x1a1a22, 0.7);
    this.scene.add(hemi);
  }

  _initFloor() {
    // Transparent shadow catcher — only the soft contact shadow is drawn.
    const geo = new THREE.PlaneGeometry(4000, 4000);
    geo.rotateX(-Math.PI / 2);
    const mat = new THREE.ShadowMaterial({ opacity: 0.28 });
    this.floor = new THREE.Mesh(geo, mat);
    this.floor.receiveShadow = true;
    this.floor.position.y = 0;
    this.scene.add(this.floor);
  }

  _initMaterial() {
    // Neutral, matte, skin-adjacent clay — not a specific skin tone.
    return new THREE.MeshPhysicalMaterial({
      color: 0xcaa596,
      roughness: 0.72,
      metalness: 0.0,
      sheen: 0.35,
      sheenColor: new THREE.Color(0xffe6d8),
      sheenRoughness: 0.6,
      clearcoat: 0.0,
      flatShading: false,
      side: THREE.DoubleSide,
    });
  }

  _initControls() {
    this.controls = new OrbitControls(this.camera, this.canvas);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.08;
    this.controls.enablePan = false;
    this.controls.rotateSpeed = 0.85;
    this.controls.autoRotate = true;
    this.controls.autoRotateSpeed = 0.55;
    // Pause the turntable while the user is actively inspecting.
    this.controls.addEventListener('start', () => { this.controls.autoRotate = false; });
    this.controls.addEventListener('end', () => { this.controls.autoRotate = true; });
  }

  _observeResize() {
    const parent = this.canvas.parentElement || this.canvas;
    const apply = () => {
      const w = Math.max(1, parent.clientWidth);
      const h = Math.max(1, parent.clientHeight);
      this.renderer.setSize(w, h, false);
      this.camera.aspect = w / h;
      this.camera.updateProjectionMatrix();
    };
    apply();
    this._resizeObserver = new ResizeObserver(apply);
    this._resizeObserver.observe(parent);
  }

  // ── Engine binding ──────────────────────────────────────────────────

  /**
   * Attach the ready engine and build the first geometry.
   * @param {object} engine  OxiHumanEngine
   * @param {WebAssembly.Memory} memory
   */
  setEngine(engine, memory) {
    this.engine = engine;
    this.memory = memory;
    this._rebuildGeometry(engine.refresh_geometry());
    this._frameCamera();
  }

  /** (Re)create the typed-array views over current WASM linear memory. */
  _makeViews() {
    const buf = this.memory.buffer;
    const e = this.engine;
    e.refresh_geometry();
    this.positions = new Float32Array(buf, e.positions_ptr(), e.positions_len());
    this.normals = new Float32Array(buf, e.normals_ptr(), e.normals_len());
    this.uvs = new Float32Array(buf, e.uvs_ptr(), e.uvs_len());
    this.indices = new Uint32Array(buf, e.indices_ptr(), e.indices_len());
  }

  /** Build a fresh BufferGeometry from the current views and swap it in. */
  _rebuildGeometry(gen) {
    this._makeViews();
    const geo = new THREE.BufferGeometry();
    const pos = new THREE.BufferAttribute(this.positions, 3);
    const nrm = new THREE.BufferAttribute(this.normals, 3);
    pos.setUsage(THREE.DynamicDrawUsage);
    nrm.setUsage(THREE.DynamicDrawUsage);
    geo.setAttribute('position', pos);
    geo.setAttribute('normal', nrm);
    if (this.uvs.length >= 2) {
      geo.setAttribute('uv', new THREE.BufferAttribute(this.uvs, 2));
    }
    geo.setIndex(new THREE.BufferAttribute(this.indices, 1));
    geo.computeBoundingBox();
    geo.computeBoundingSphere();

    const old = this.geometry;
    this.geometry = geo;
    this.gen = gen;

    if (!this.mesh) {
      this.mesh = new THREE.Mesh(geo, this._initMaterial());
      this.mesh.castShadow = true;
      this.mesh.receiveShadow = false;
      this.scene.add(this.mesh);
    } else {
      this.mesh.geometry = geo;
    }
    if (old && old !== geo) old.dispose();

    const size = new THREE.Vector3();
    geo.boundingBox.getSize(size);
    this.figureHeight = Math.max(size.y, 1e-3);
    // Keep the shadow catcher glued to the soles.
    this.floor.position.y = geo.boundingBox.min.y - this.figureHeight * 0.002;
    this._aimKeyLight(geo.boundingBox);
  }

  /** Point the key light's shadow frustum at the figure. */
  _aimKeyLight(box) {
    const center = new THREE.Vector3();
    box.getCenter(center);
    const size = new THREE.Vector3();
    box.getSize(size);
    const r = Math.max(size.x, size.y, size.z) * 0.75 + 1;
    const cam = this.keyLight.shadow.camera;
    cam.left = -r; cam.right = r; cam.top = r; cam.bottom = -r;
    cam.near = 0.1; cam.far = r * 8;
    cam.updateProjectionMatrix();
    this.keyLight.target.position.copy(center);
    this.keyLight.position.set(center.x + r * 0.6, box.max.y + r * 0.5, center.z + r * 0.9);
  }

  /** Frame the camera on the current figure (called once at load). */
  _frameCamera() {
    const box = this.geometry.boundingBox;
    const size = new THREE.Vector3();
    const center = new THREE.Vector3();
    box.getSize(size);
    box.getCenter(center);
    const maxDim = Math.max(size.x, size.y, size.z);
    const fov = (this.camera.fov * Math.PI) / 180;
    let dist = (maxDim / 2) / Math.tan(fov / 2);
    dist *= 1.55;
    this.controls.target.copy(center);
    this.camera.position.set(center.x + dist * 0.12, center.y + size.y * 0.06, center.z + dist);
    this.camera.near = Math.max(dist / 500, 0.001);
    this.camera.far = dist * 500;
    this.camera.updateProjectionMatrix();
    this.controls.minDistance = dist * 0.35;
    this.controls.maxDistance = dist * 3.5;
    this.controls.update();
  }

  /** Flag a geometry refresh for the next frame (called on param change). */
  requestMorph() { this.pendingMorph = true; }

  _morph() {
    const g = this.engine.refresh_geometry();
    if (g !== this.gen || this.positions.buffer !== this.memory.buffer) {
      this._rebuildGeometry(g);
    } else {
      this.geometry.attributes.position.needsUpdate = true;
      this.geometry.attributes.normal.needsUpdate = true;
      this.geometry.computeBoundingSphere();
    }
  }

  _breathe(tSec) {
    if (!this.mesh) return;
    const b = Math.sin(tSec * 1.7);
    // Render-level only: a subtle chest/depth expansion + ribcage rise.
    this.mesh.scale.set(1 + b * 0.011, 1 + b * 0.004, 1 + b * 0.011);
  }

  _tickFps(now) {
    if (this._lastNow) {
      const dt = now - this._lastNow;
      this._frameTimes.push(dt);
      if (this._frameTimes.length > 90) this._frameTimes.shift();
    }
    this._lastNow = now;
  }

  get fps() {
    if (!this._frameTimes.length) return 0;
    let s = 0;
    for (const d of this._frameTimes) s += d;
    const avg = s / this._frameTimes.length;
    return avg > 0 ? Math.round(1000 / avg) : 0;
  }

  /** Single animation step. `now` is a performance.now() timestamp (ms). */
  render(now) {
    if (this.engine) {
      // Guard: an export or fit may have grown WASM memory and detached views.
      if (this.positions.buffer !== this.memory.buffer) {
        this._rebuildGeometry(this.engine.mesh_generation());
      }
      if (this.pendingMorph) {
        this._morph();
        this.pendingMorph = false;
      }
      this._breathe(now / 1000);
    }
    this.controls.update();
    this.renderer.render(this.scene, this.camera);
    this._tickFps(now);
  }
}
