extern crate sapling_crypto;
extern crate wasm_bindgen;
extern crate bellman;
extern crate pairing;

#[macro_use]
extern crate serde_derive;

use wasm_bindgen::prelude::*;

use bellman::{
    Circuit,
    SynthesisError,
    ConstraintSystem,
    groth16::{Proof, Parameters, verify_proof, create_random_proof, prepare_verifying_key, generate_random_parameters}
};

use rand::{XorShiftRng, SeedableRng};
use ff::{Field, PrimeField};
use pairing::{Engine, bls12_381::{Bls12, Fr, FrRepr}};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

struct MySillyCircuit<E: Engine> {
    a: Option<E::Fr>,
    b: Option<E::Fr>
}

impl<E: Engine> Circuit<E> for MySillyCircuit<E> {
    fn synthesize<CS: ConstraintSystem<E>>(
        self,
        cs: &mut CS
    ) -> Result<(), SynthesisError>
    {
        let a = cs.alloc(|| "a", || self.a.ok_or(SynthesisError::AssignmentMissing))?;
        let b = cs.alloc(|| "b", || self.b.ok_or(SynthesisError::AssignmentMissing))?;
        let c = cs.alloc_input(|| "c", || {
            let mut a = self.a.ok_or(SynthesisError::AssignmentMissing)?;
            let b = self.b.ok_or(SynthesisError::AssignmentMissing)?;

            a.mul_assign(&b);
            Ok(a)
        })?;

        cs.enforce(
            || "a*b=c",
            |lc| lc + a,
            |lc| lc + b,
            |lc| lc + c
        );

        Ok(())
    }
}

#[derive(Serialize)]
pub struct KGGenerate {
    pub params: String
}

#[derive(Serialize)]
pub struct KGProof {
    pub proof: String
}

#[derive(Serialize)]
pub struct KGVerify {
    pub result: bool
}

#[wasm_bindgen]
pub fn generate() -> JsValue {
    let rng = &mut XorShiftRng::from_seed([0x5dbe6259, 0x8d313d76, 0x3237db17, 0xe5bc0654]);

    let params = generate_random_parameters::<Bls12, _, _>(
        MySillyCircuit { a: None, b: None },
        rng
    ).unwrap();

    let mut v = vec![];

    params.write(&mut v).unwrap();
    assert_eq!(v.len(), 2136);

    JsValue::from_serde(&KGGenerate {
        params: hex::encode(&v[..])
    }).unwrap()
}

#[wasm_bindgen]
pub fn prove(params: &str, a_raw: u32, b_raw: u32) -> JsValue {
    let de_params = Parameters::<Bls12>::read(&hex::decode(params).unwrap()[..], true).unwrap();

    let a = Fr::from_repr(FrRepr::from(a_raw as u64)).unwrap();
    let b = Fr::from_repr(FrRepr::from(b_raw as u64)).unwrap();
    let mut c = a;
    c.mul_assign(&b);

    let rng = &mut XorShiftRng::from_seed([0x5dbe6259, 0x8d313d76, 0x3237db17, 0xe5bc0654]);
    let proof = create_random_proof(
        MySillyCircuit {
            a: Some(a),
            b: Some(b)
        },
        &de_params,
        rng
    ).unwrap();

    let mut v = vec![];
    proof.write(&mut v).unwrap();

    JsValue::from_serde(&KGProof {
        proof: hex::encode(&v[..])
    }).unwrap()
}

#[wasm_bindgen]
pub fn verify(params: &str, proof: &str, c: u32) -> JsValue {
    let de_params = Parameters::read(&hex::decode(params).unwrap()[..], true).unwrap();
    let pvk = prepare_verifying_key::<Bls12>(&de_params.vk);
    let result = verify_proof(&pvk, &Proof::read(&hex::decode(proof).unwrap()[..]).unwrap(), &[Fr::from_repr(FrRepr::from(c as u64)).unwrap()]).unwrap();

    JsValue::from_serde(&KGVerify{
        result: result
    }).unwrap()
}
