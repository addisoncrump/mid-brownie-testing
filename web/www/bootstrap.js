init();

async function init() {
  if (typeof process == "object") {
    // We run in the npm/webpack environment.
    const [{ Chart }, { main, setup }] = await Promise.all([
      import("wasm-demo"),
      import("./index.js"),
    ]);
    setup(Chart);
    main();
  } else {
    const [{ Chart, default: init, initThreadPool }, { main, setup }] =
      await Promise.all([import("../pkg/web.js"), import("./index.js")]);
    // Regular wasm-bindgen initialization.
    await init();

    // Thread pool initialization with the given number of threads
    // (pass `navigator.hardwareConcurrency` if you want to use all cores).
    await initThreadPool(navigator.hardwareConcurrency);
    setup(Chart);
    main();
  }
}
