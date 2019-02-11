extern crate sapling_crypto;
extern crate wasm_bindgen;
extern crate bellman;
extern crate pairing;

#[macro_use]
extern crate serde_derive;

use std::error::Error;

use wasm_bindgen::prelude::*;

use num_bigint::BigInt;
use num_traits::Num;

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
        )?;

        h.inputize(cs)?;

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

#[wasm_bindgen(catch)]
pub fn generate(seed_slice: &[u32]) -> Result<JsValue, JsValue> {
    let res = || -> Result<JsValue, Box<Error>> {
        let mut seed : [u32; 4] = [0; 4];
        seed.copy_from_slice(seed_slice);
        let rng = &mut XorShiftRng::from_seed(seed);

        let j_params = &JubjubBn256::new();
        let params = generate_random_parameters::<Bn256, _, _>(
            DiscreteLogCircuit {
                params: j_params,
                x: None
            },
            rng
        )?;

        let mut v = vec![];

        params.write(&mut v)?;

        Ok(JsValue::from_serde(&KGGenerate {
            params: hex::encode(&v[..])
        })?)
    }();
    convert_error_to_jsvalue(res)
}

#[wasm_bindgen(catch)]
pub fn prove(seed_slice: &[u32], params: &str, x_hex: &str) -> Result<JsValue, JsValue> {
    let res = || -> Result<JsValue, Box<Error>> {
        if params.len() == 0 {
            return Err("Params are empty. Did you generate or load params?".into())
        }
        let de_params = Parameters::<Bn256>::read(&hex::decode(params)?[..], true)?;

        let mut seed : [u32; 4] = [0; 4];
        seed.copy_from_slice(seed_slice);
        let rng = &mut XorShiftRng::from_seed(seed);
        let params = &JubjubBn256::new();

        let g = params.generator(FixedGenerators::ProofGenerationKey);
        let s = &format!("{}", Fs::char())[2..];
        let s_big = BigInt::from_str_radix(s, 16)?;
        let x_big = BigInt::from_str_radix(x_hex, 16)?;
        if x_big >= s_big {
            return Err("x should be less than 60c89ce5c263405370a08b6d0302b0bab3eedb83920ee0a677297dc392126f1".into())
        }
        let x_raw = &x_big.to_str_radix(10);
        let x = Fr::from_str(x_raw).ok_or("couldn't parse Fr")?;

        let xs = Fs::from_str(x_raw).ok_or("couldn't parse Fr")?;

        let h = g.mul(xs, params);

        let proof = create_random_proof(
            DiscreteLogCircuit {
                params: params,
                x: Some(x),
            },
            &de_params,
            rng
        )?;

        let mut v = vec![];
        proof.write(&mut v)?;

        let mut v2 = vec![];
        h.write(&mut v2)?;

        Ok(JsValue::from_serde(&KGProof {
            proof: hex::encode(&v[..]),
            h: hex::encode(&v2[..])
        })?)
    }();

    convert_error_to_jsvalue(res)
}

#[wasm_bindgen(catch)]
pub fn verify(params: &str, proof: &str, h: &str) -> Result<JsValue, JsValue> {
    let res = || -> Result<JsValue, Box<Error>> {
        let j_params = &JubjubBn256::new();
        let de_params = Parameters::read(&hex::decode(params)?[..], true)?;
        let pvk = prepare_verifying_key::<Bn256>(&de_params.vk);
        let h = Point::<Bn256, _>::read(&hex::decode(h)?[..], j_params)?;
        let (h_x, h_y) = h.into_xy();
        let result = verify_proof(
            &pvk,
            &Proof::read(&hex::decode(proof)?[..])?,
            &[
            h_x,
            h_y
            ])?;

        Ok(JsValue::from_serde(&KGVerify{
            result: result
        })?)
    }();
    convert_error_to_jsvalue(res)
}

fn convert_error_to_jsvalue(res: Result<JsValue, Box<Error>>) -> Result<JsValue, JsValue> {
    if res.is_ok() {
        Ok(res.ok().unwrap())
    } else {
        Err(JsValue::from_str(&res.err().unwrap().to_string()))
    }
}


#[cfg(test)]
mod test {
    use rand::{XorShiftRng, SeedableRng, Rng};
    use pairing::{bn256::{Bn256, Fr}};
    use sapling_crypto::{
        babyjubjub::{
            fs::Fs,
            JubjubBn256,
            FixedGenerators,
            JubjubEngine,
            JubjubParams,
            edwards::Point
        }
    };
    use sapling_crypto::circuit::boolean::{Boolean, AllocatedBit};
    use sapling_crypto::circuit::test::TestConstraintSystem;
    use bellman::{
        Circuit,
        SynthesisError,
        ConstraintSystem,
        groth16::{Proof, Parameters, verify_proof, create_random_proof, prepare_verifying_key, generate_random_parameters}
    };

    use super::DiscreteLogCircuit;

    #[test]
    fn print_g() {
        let j_params = &JubjubBn256::new();
        let g = j_params.generator(FixedGenerators::ProofGenerationKey);
        println!("{}, {}", g.into_xy().0, g.into_xy().1);
    }

    #[test]
    fn print_debug_info() {
        let mut cs = TestConstraintSystem::<Bn256>::new();
        let rng = &mut XorShiftRng::from_seed([0x3dbe6259, 0x8d313d76, 0x3237db17, 0xe5bc0654]);
        let j_params = &JubjubBn256::new();

        let dl = DiscreteLogCircuit {
            params: j_params,
            x: None,
        };
        dl.synthesize(&mut cs).unwrap();
        println!("num constraints: {}", cs.num_constraints());
        println!("num inputs: {}", cs.num_inputs());
        println!("num aux: {}", cs.num_aux());
    }

}
