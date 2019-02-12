#![feature(duration_as_u128)]

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

#[cfg(not(target_arch = "wasm32"))]
use std::time::{Instant};

use bellman::{
    Circuit,
    SynthesisError,
    ConstraintSystem,
    groth16::{Proof, Parameters, verify_proof, create_random_proof, prepare_verifying_key, generate_random_parameters}
};

pub struct Stopwatch {
    #[cfg(not(target_arch = "wasm32"))]
    start: Instant,
    #[cfg(target_arch = "wasm32")]
    timer_id: u32
}

impl Stopwatch {

    pub fn start() -> Self {
        Stopwatch {
            #[cfg(not(target_arch = "wasm32"))]
            start: Instant::now(),
            #[cfg(target_arch = "wasm32")]
            timer_id: start_timer()
        }
    }

    pub fn finish(self) -> u128 {
        #[cfg(not(target_arch = "wasm32"))]
        return self.start.elapsed().as_millis();

        #[cfg(target_arch = "wasm32")]
        return finish_timer(self.timer_id) as u128;
    }
}


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

#[wasm_bindgen(module = "./helpers")]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    fn start_timer() -> u32;
    fn finish_timer(timer_id: u32) -> u32;
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
    pub params: String,
    pub millis: u128
}

#[derive(Serialize)]
pub struct KGProof {
    pub proof: String,
    pub h: String,
    pub millis: u128
}

#[derive(Serialize)]
pub struct KGVerify {
    pub result: bool,
    pub millis: u128

}

#[wasm_bindgen(catch)]
pub fn generate(seed_slice: &[u32]) -> Result<JsValue, JsValue> {
    let res = run_generate(seed_slice);
    if res.is_ok() {
        Ok(JsValue::from_serde(&res.ok().unwrap()).unwrap())
    } else {
        Err(JsValue::from_str(&res.err().unwrap().to_string()))
    }
}

fn run_generate(seed_slice: &[u32]) -> Result<KGGenerate, Box<Error>> {
    let mut seed : [u32; 4] = [0; 4];
    seed.copy_from_slice(seed_slice);
    let rng = &mut XorShiftRng::from_seed(seed);

    let stopwatch = Stopwatch::start();
    let j_params = &JubjubBn256::new();
    let params = generate_random_parameters::<Bn256, _, _>(
        DiscreteLogCircuit {
            params: j_params,
            x: None
        },
        rng
    )?;
    let millis = stopwatch.finish();

    let mut v = vec![];

    params.write(&mut v)?;

    Ok(KGGenerate {
        params: hex::encode(&v[..]),
        millis: millis
    })
}

#[wasm_bindgen(catch)]
pub fn prove(seed_slice: &[u32], params: &str, x_hex: &str) -> Result<JsValue, JsValue> {
    let res = run_prove(seed_slice, params, x_hex);
    if res.is_ok() {
        Ok(JsValue::from_serde(&res.ok().unwrap()).unwrap())
    } else {
        Err(JsValue::from_str(&res.err().unwrap().to_string()))
    }
}

fn run_prove(seed_slice: &[u32], params: &str, x_hex: &str) -> Result<KGProof, Box<Error>> {
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

    let stopwatch = Stopwatch::start();
    let h = g.mul(xs, params);

    let proof = create_random_proof(
        DiscreteLogCircuit {
            params: params,
            x: Some(x),
        },
        &de_params,
        rng
    )?;
    let millis = stopwatch.finish();

    let mut v = vec![];
    proof.write(&mut v)?;

    let mut v2 = vec![];
    h.write(&mut v2)?;

    Ok(KGProof {
        proof: hex::encode(&v[..]),
        h: hex::encode(&v2[..]),
        millis: millis
    })
}

#[wasm_bindgen(catch)]
pub fn verify(params: &str, proof: &str, h: &str) -> Result<JsValue, JsValue> {
    let res = run_verify(params, proof, h);
    if res.is_ok() {
        Ok(JsValue::from_serde(&res.ok().unwrap()).unwrap())
    } else {
        Err(JsValue::from_str(&res.err().unwrap().to_string()))
    }

}

fn run_verify(params: &str, proof: &str, h: &str) -> Result<KGVerify, Box<Error>> {
    let j_params = &JubjubBn256::new();
    let de_params = Parameters::read(&hex::decode(params)?[..], true)?;
    let pvk = prepare_verifying_key::<Bn256>(&de_params.vk);
    let h = Point::<Bn256, _>::read(&hex::decode(h)?[..], j_params)?;

    let stopwatch = Stopwatch::start();
    let (h_x, h_y) = h.into_xy();
    let result = verify_proof(
        &pvk,
        &Proof::read(&hex::decode(proof)?[..])?,
        &[
        h_x,
        h_y
        ])?;

    let millis = stopwatch.finish();
    Ok(KGVerify{
        result: result,
        millis: millis
    })
}

#[cfg(test)]
mod test {
    use pairing::{bn256::{Bn256}};
    use sapling_crypto::{
        babyjubjub::{
            JubjubBn256,
            FixedGenerators,
            JubjubParams,
        }
    };
    use sapling_crypto::circuit::test::TestConstraintSystem;
    use bellman::{
        Circuit,
    };

    use super::DiscreteLogCircuit;
    use std::fs;

    #[test]
    fn print_g() {
        let j_params = &JubjubBn256::new();
        let g = j_params.generator(FixedGenerators::ProofGenerationKey);
        println!("{}, {}", g.into_xy().0, g.into_xy().1);
    }

    #[test]
    fn print_debug_info() {
        let mut cs = TestConstraintSystem::<Bn256>::new();
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

    #[test]
    fn time_generate() {
        use super::run_generate;

        let params = run_generate(&[1,2,3,4]).unwrap();
        println!("generate time elapsed: {}", params.millis);
        //fs::write("test/test.params", params.params);
    }

    #[test]
    fn time_prove() {
        use super::run_prove;
        let params = &String::from_utf8(fs::read("test/test.params").unwrap()).unwrap();
        let proof = run_prove(&[1,2,3,4], params, "5").unwrap();
        //fs::write("test/test.proof", proof.proof);
        //fs::write("test/test.h", proof.h);
        println!("prove time elapsed: {}", proof.millis);
    }

    #[test]
    fn time_verify() {
        use super::run_verify;
        let params = &String::from_utf8(fs::read("test/test.params").unwrap()).unwrap();
        let proof = &String::from_utf8(fs::read("test/test.proof").unwrap()).unwrap();
        let h = &String::from_utf8(fs::read("test/test.h").unwrap()).unwrap();

        let verify = run_verify(params, proof, h).unwrap();
        println!("verify: {}", verify.result);
        println!("verify time elapsed: {}", verify.millis);
    }
}
