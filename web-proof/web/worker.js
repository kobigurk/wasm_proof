let js;

import("./web_proof").then(loaded => {
  js = loaded;
  postMessage({type: 'wasm_loaded'});
});

onmessage = event => {
  try {
    if (!js) {
      throw new Error('Wasm module not loaded yet.');
    }
    switch (event.data.type) {
      case 'generate':
        var seed = new Uint32Array(4);
        self.crypto.getRandomValues(seed);

        var gen = js.generate(seed);
        postMessage({type: event.data.type, circuit: event.data.circuit, result: gen});
        break;
      case 'prove':
        var seed = new Uint32Array(4);
        self.crypto.getRandomValues(seed);
        var p = js.prove(seed, event.data.params, event.data.x);
        postMessage({type: event.data.type, circuit: event.data.circuit, result: p});
        break;
      case 'verify':
        var v = js.verify(event.data.params, event.data.proof, event.data.h);
        postMessage({type: event.data.type, circuit: event.data.circuit, result: v});
        break;
    }
  } catch(e) {
    postMessage({type: event.data.type, circuit: event.data.circuit, error: e});
  }
};

