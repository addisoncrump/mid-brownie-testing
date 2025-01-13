// If you only use `npm` you can simply
// import { Chart } from "wasm-demo" and remove `setup` call from `bootstrap.js`.
class Chart {
}

const canvas = document.getElementById("canvas");
const pitch = document.getElementById("pitch");
const yaw = document.getElementById("yaw");
const iterations = document.getElementById("iterations");
const seed = document.getElementById("seed");
const noise = document.getElementById("noise");
const decay = document.getElementById("decay");
const status = document.getElementById("status");
const bound = document.getElementById("bound");

const render = document.getElementById("render");
const renderStatus = document.getElementById("renderStatus");

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
    renderStatus.innerText = "WebAssembly loaded!";
    yaw.addEventListener("change", updatePlot);
    pitch.addEventListener("change", updatePlot);
    iterations.addEventListener("change", updatePlot);
    bound.addEventListener("change", updatePlot);
    yaw.addEventListener("input", updatePlot);
    pitch.addEventListener("input", updatePlot);
    iterations.addEventListener("input", updatePlot);
    bound.addEventListener("input", updatePlot);

    seed.addEventListener("change", setupCanvas);
    noise.addEventListener("change", setupCanvas);
    decay.addEventListener("change", setupCanvas);
    seed.addEventListener("input", setupCanvas);
    noise.addEventListener("input", setupCanvas);
    decay.addEventListener("input", setupCanvas);
    window.addEventListener("resize", setupCanvas);
}

function fixCanvasWidths(actual) {
    const dpr = window.devicePixelRatio || 1.0;
    const size = actual.parentNode.offsetWidth;
    const aspectRatio = actual.width / actual.height;
    actual.style.width = size + "px";
    actual.style.height = size / aspectRatio + "px";
    actual.width = dpr * size;
    actual.height = dpr * size / aspectRatio;
}

/** Setup canvas to properly handle high DPI and redraw current plot. */
function setupCanvas() {
    fixCanvasWidths(canvas);
    // fixCanvasWidths(render);

    const seed_value = BigInt(seed.value)
    const noise_value = Number(noise.value)
    const decay_value = Number(decay.value) / decay.max
    chart = Chart.new(noise_value, decay_value, seed_value)
    updatePlot();
    // updateRender();
}

function updatePlot3d() {
    const context = canvas.getContext('2d');
    context.clearRect(0, 0, canvas.width, canvas.height);
    let bound_value = Boolean(bound.checked);
    let pitch_value = Number(pitch.value) / 100.0;
    let yaw_value = Number(yaw.value) / 100.0;
    let iterations_value = Number(iterations.value);
    chart.plot3d(canvas, bound_value, pitch_value, yaw_value, iterations_value);
}

function updateRender3d() {
    const context = render.getContext('2d');
    context.clearRect(0, 0, canvas.width, canvas.height);
    chart.render3d(canvas);
}

/** Redraw currently selected plot. */
function updatePlot() {
    status.innerText = `Rendering 3d plot...`;
    const start = performance.now();

    updatePlot3d();

    const end = performance.now();
    status.innerText = `Rendered 3d plot in ${Math.ceil(end - start)}ms`;
}

/** Redraw currently selected plot. */
function updateRender() {
    renderStatus.innerText = `Rendering camera view...`;
    const start = performance.now();

    updateRender3d();

    const end = performance.now();
    renderStatus.innerText = `Rendered camera view in ${Math.ceil(end - start)}ms`;
}
