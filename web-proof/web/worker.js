let js;

import("./web_proof").then(loaded => {
  js = loaded;
});

onmessage = event => {
  try {
    switch (event.data.type) {
      case 'generate':
        var gen = js.generate();
        postMessage({type: event.data.type, result: gen});
        break;
      case 'prove':
        var p = js.prove(event.data.params, event.data.x);
        postMessage({type: event.data.type, result: p});
        break;
      case 'verify':
        var v = js.verify(event.data.params, event.data.proof, event.data.h);
        postMessage({type: event.data.type, result: v});
        break;
    }
  } catch(e) {
    postMessage({type: event.data.type, error: e.message});
  }
};

