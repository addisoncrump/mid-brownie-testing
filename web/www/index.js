// If you only use `npm` you can simply
// import { Chart } from "wasm-demo" and remove `setup` call from `bootstrap.js`.
class Chart {}

const canvas = document.getElementById("canvas");
const pitch = document.getElementById("pitch");
const yaw = document.getElementById("yaw");
const iterations = document.getElementById("iterations");
const status = document.getElementById("status");

let chart = null;

/** Main entry point */
export function main() {
  setupUI();
  setupCanvas();
}

/** This function is used in `bootstrap.js` to setup imports. */
export function setup(WasmChart) {
  Chart = WasmChart;
}

/** Add event listeners. */
function setupUI() {
  status.innerText = "WebAssembly loaded!";
  yaw.addEventListener("change", () => requestAnimationFrame(updatePlot));
  pitch.addEventListener("change", () => requestAnimationFrame(updatePlot));
  iterations.addEventListener("change", () =>
    requestAnimationFrame(updatePlot)
  );
  yaw.addEventListener("input", () => requestAnimationFrame(updatePlot));
  pitch.addEventListener("input", () => requestAnimationFrame(updatePlot));
  iterations.addEventListener("input", () => requestAnimationFrame(updatePlot));
  window.addEventListener("resize", setupCanvas);
}

/** Setup canvas to properly handle high DPI and redraw current plot. */
function setupCanvas() {
  const dpr = window.devicePixelRatio || 1.0;
  const aspectRatio = canvas.width / canvas.height;
  const size = canvas.parentNode.offsetWidth * 0.8;
  canvas.style.width = size + "px";
  canvas.style.height = size / aspectRatio + "px";
  canvas.width = size;
  canvas.height = size / aspectRatio;
  chart = Chart.new(BigInt(10000), 0.5, BigInt(1));
  iterations.value = 1;
  updatePlot();
}

function updatePlot3d() {
  let yaw_value = Number(yaw.value) / 100.0;
  let pitch_value = Number(pitch.value) / 100.0;
  let iterations_value = Number(iterations.value);
  chart.plot3d(canvas, pitch_value, yaw_value, iterations_value);
}

/** Redraw currently selected plot. */
function updatePlot() {
  status.innerText = `Rendering 3d plot...`;
  const start = performance.now();

  updatePlot3d();

  const end = performance.now();
  status.innerText = `Rendered 3d plot in ${Math.ceil(end - start)}ms`;
}
