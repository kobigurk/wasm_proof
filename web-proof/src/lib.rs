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
use ff::{BitIterator, PrimeField};
use pairing::{bn256::{Bn256, Fr}};
use sapling_crypto::{
    babyjubjub::{
        fs::Fs,
        JubjubBn256,
        FixedGenerators,
        JubjubEngine,
        JubjubParams,
        edwards::Point
    },
    circuit::{
        baby_ecc::fixed_base_multiplication,
        boolean::{AllocatedBit, Boolean}
    }
};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

struct DiscreteLogCircuit<'a, E: JubjubEngine> {
    pub params: &'a E::Params,
    pub x: Option<E::Fr>,
}

impl<'a, E: JubjubEngine> Circuit<E> for DiscreteLogCircuit<'a, E> {
    fn synthesize<CS: ConstraintSystem<E>>(
        self,
        cs: &mut CS
    ) -> Result<(), SynthesisError>
    {
        let mut x_bits = match self.x {
            Some(x) => {
                BitIterator::new(x.into_repr()).collect::<Vec<_>>()
            }
            None => {
                vec![false; Fs::NUM_BITS as usize]
            }
        };

        x_bits.reverse();
        x_bits.truncate(Fs::NUM_BITS as usize);

        let x_bits = x_bits.into_iter()
                           .enumerate()
                           .map(|(i, b)| AllocatedBit::alloc(cs.namespace(|| format!("scalar bit {}", i)), Some(b)).unwrap())
                           .map(|v| Boolean::from(v))
                           .collect::<Vec<_>>();


        let h = fixed_base_multiplication(
            cs.namespace(|| "multiplication"),
            FixedGenerators::ProofGenerationKey,
            &x_bits,
            self.params
        ).unwrap();

        h.inputize(cs).unwrap();

        Ok(())
    }
}

#[derive(Serialize)]
pub struct KGGenerate {
    pub params: String
}

#[derive(Serialize)]
pub struct KGProof {
    pub proof: String,
    pub h: String
}

#[derive(Serialize)]
pub struct KGVerify {
    pub result: bool
}

#[wasm_bindgen]
pub fn generate() -> JsValue {
    let rng = &mut XorShiftRng::from_seed([0x5dbe6259, 0x8d313d76, 0x3237db17, 0xe5bc0654]);

    let j_params = &JubjubBn256::new();
    let params = generate_random_parameters::<Bn256, _, _>(
        DiscreteLogCircuit {
            params: j_params,
            x: None
        },
        rng
    ).unwrap();

    let mut v = vec![];

    params.write(&mut v).unwrap();

    JsValue::from_serde(&KGGenerate {
        params: hex::encode(&v[..])
    }).unwrap()
}

#[wasm_bindgen]
pub fn prove(params: &str, x_raw: &str) -> JsValue {
    let de_params = Parameters::<Bn256>::read(&hex::decode(params).unwrap()[..], true).unwrap();

    let rng = &mut XorShiftRng::from_seed([0x5dbe6259, 0x8d313d76, 0x3237db17, 0xe5bc0654]);
    let params = &JubjubBn256::new();

    let g = params.generator(FixedGenerators::ProofGenerationKey);
    let x = Fr::from_str(x_raw).unwrap();

    let xs = Fs::from_str(x_raw).unwrap();

    let h = g.mul(xs, params);

    let proof = create_random_proof(
        DiscreteLogCircuit {
            params: params,
            x: Some(x),
        },
        &de_params,
        rng
    ).unwrap();

    let mut v = vec![];
    proof.write(&mut v).unwrap();

    let mut v2 = vec![];
    h.write(&mut v2).unwrap();

    JsValue::from_serde(&KGProof {
        proof: hex::encode(&v[..]),
        h: hex::encode(&v2[..])
    }).unwrap()
}

#[wasm_bindgen]
pub fn verify(params: &str, proof: &str, h: &str) -> JsValue {
    let j_params = &JubjubBn256::new();
    let de_params = Parameters::read(&hex::decode(params).unwrap()[..], true).unwrap();
    let pvk = prepare_verifying_key::<Bn256>(&de_params.vk);
    let h = Point::<Bn256, _>::read(&hex::decode(h).unwrap()[..], j_params).unwrap();
    let (h_x, h_y) = h.into_xy();
    let result = verify_proof(
        &pvk,
        &Proof::read(&hex::decode(proof).unwrap()[..]).unwrap(),
        &[
        h_x,
        h_y
        ]).unwrap();

    JsValue::from_serde(&KGVerify{
        result: result
    }).unwrap()
}
