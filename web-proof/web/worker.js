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
        var seed = new Uint32Array(8);
        self.crypto.getRandomValues(seed);

        var gen;
        switch (event.data.circuit) {
          case 'dl':
            gen = js.generate(seed);
            break;
          case 'tree':
            gen = js.generate_tree(seed, event.data.depth);
            break;

        }
        postMessage({type: event.data.type, circuit: event.data.circuit, result: gen});
        break;
      case 'prove':
        var seed = new Uint32Array(8);
        self.crypto.getRandomValues(seed);
        var p;
        switch (event.data.circuit) {
          case 'dl':
            p = js.prove(seed, event.data.params, event.data.x);
            break;
          case 'tree':
            p = js.prove_tree(seed, event.data.params, event.data.x, event.data.depth);
            break;

        }
        postMessage({type: event.data.type, circuit: event.data.circuit, result: p});
        break;
      case 'verify':
        var v;
        switch (event.data.circuit) {
          case 'dl':
            v = js.verify(event.data.params, event.data.proof, event.data.h);
            break;

          case 'tree':
            v = js.verify_tree(event.data.params, event.data.proof, event.data.h);
            break;
        }

        postMessage({type: event.data.type, circuit: event.data.circuit, result: v});
        break;
    }
  } catch(e) {
    postMessage({type: event.data.type, circuit: event.data.circuit, error: e});
  }
};

