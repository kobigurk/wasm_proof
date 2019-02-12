# ZKP in WebAssembly

Online version: [https://zkwasm.kobi.one](https://zkwasm.kobi.one)

Discussion: [https://community.zkproof.org/t/zksnarks-in-webassembly-running-demo-and-discussion/30](https://community.zkproof.org/t/zksnarks-in-webassembly-running-demo-and-discussion/30)

| Circuit                  | Num Constraints | CPU      | Platform  | Phase    | Running time (milliseconds) |
| ------------------------ | --------------- |--------- | --------- | -------- | --------------------------- |
| Discrete Log             | 1085            | i7-7500U | x86\_64   | Generate | 881                         |
| Discrete Log             | 1085            | i7-7500U | x86\_64   | Prove    | 169                         |
| Discrete Log             | 1085            | i7-7500U | x86\_64   | Verify   | 5                           |
| Merkle Tree (depth 32)   | 44193           | i7-7500U | x86\_64   | Generate | 6949                        |
| Merkle Tree (depth 32)   | 44193           | i7-7500U | x86\_64   | Prove    | 831                         |
| Merkle Tree (depth 32)   | 44193           | i7-7500U | x86\_64   | Verify   | 5                           |
| Discrete Log             | 1085            | i7-7500U | WASM      | Generate | 3785                        |
| Discrete Log             | 1085            | i7-7500U | WASM      | Prove    | 606                         |
| Discrete Log             | 1085            | i7-7500U | WASM      | Verify   | 16                          |
| Merkle Tree (depth 32)   | 44193           | i7-7500U | WASM      | Generate | 130047                      |
| Merkle Tree (depth 32)   | 44193           | i7-7500U | WASM      | Prove    | 14926                       |
| Merkle Tree (depth 32)   | 44193           | i7-7500U | WASM      | Verify   | 16                          |
