use anyhow::{bail};
#[macro_use]
extern crate load_file;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate hex_literal;
extern crate bellman_ce;
extern crate byteorder;
extern crate itertools;
extern crate num_bigint;
extern crate num_traits;
extern crate rand;

pub mod circom_circuit;
pub mod plonk;
pub mod r1cs_file;
pub mod reader;
pub mod transpile;
pub mod utils;

use std::fs::File;
use std::io::{BufReader, Read};
use bytebuffer::{ByteBuffer};
use std::result::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use bellman_ce::{
    Field, PrimeFieldRepr,
    pairing::{
        bn256::Bn256,
        ff::{PrimeField},
        Engine,
    },
    kate_commitment::{Crs, CrsForMonomialForm},
    plonk::{
        better_cs::cs::PlonkCsWidth4WithNextStepParams,
        better_cs::keys::{Proof, VerificationKey},
    },
};


pub fn load_witness_from_u8_arr<E: Engine, R: Read>(mut f: R) -> Result<Vec<E::Fr>, anyhow::Error> {
    let mut buffer = [0u8; 4];
    let _wtns_header = f.read(&mut buffer[..]);
    if buffer != [119, 116, 110, 115] {
        bail!("Invalid witness header");
    }
    let version = f.read_u32::<LittleEndian>().ok().unwrap();
    if version != 2 {
        bail!("");
    }
    let num_sections = f.read_u32::<LittleEndian>().ok().unwrap();
    if num_sections != 2 {
        bail!("");
    }
    let sec_type = f.read_u32::<LittleEndian>().ok().unwrap();
    if sec_type != 1 {
        bail!("");
    }
    let sec_size = f.read_u64::<LittleEndian>().ok().unwrap();
    if sec_size != 40 {
        bail!("");
    }
    let field_size = f.read_u32::<LittleEndian>().ok().unwrap();
    if field_size != 32 {
        bail!("");
    }

    let mut prime = vec![0u8; 32];
    f.read_exact(&mut prime).ok().unwrap();
    if prime != hex!("010000f093f5e1439170b97948e833285d588181b64550b829a031e1724e6430") {
        bail!("");
    }

    let witness_len = f.read_u32::<LittleEndian>().ok().unwrap();

    let sec_type = f.read_u32::<LittleEndian>().ok().unwrap();
    if sec_type != 2 {
        bail!("");
    }
    let sec_size = f.read_u64::<LittleEndian>().ok().unwrap();
    if sec_size != (witness_len * field_size) as u64 {
        bail!("");
    }

    let mut result = Vec::with_capacity(witness_len as usize);
    for _ in 0..witness_len {
        let mut repr = E::Fr::zero().into_repr();
        repr.read_le(&mut f).ok().unwrap();
        result.push(E::Fr::from_repr(repr).ok().unwrap());
    }
    Ok(result)
}

pub fn load_key_monomial_form_from_u8_arr<E: Engine>(mut buf: BufReader::<ByteBuffer>) -> Crs<E, CrsForMonomialForm> {
    Crs::<E, CrsForMonomialForm>::read(&mut buf).expect("read key_monomial_form err")
}
pub fn test_plonk() {
    let r1cs_data: &[u8] = load_bytes!("../circuits/Test2.r1cs");
    let r1cs_byte_buf = ByteBuffer::from_bytes(r1cs_data);
    let r1cs_file = r1cs_file::from_reader::<BufReader<ByteBuffer>>(BufReader::new(r1cs_byte_buf)).ok().unwrap();
    let num_inputs = (1 + r1cs_file.header.n_pub_in + r1cs_file.header.n_pub_out) as usize;
    let num_variables = r1cs_file.header.n_wires as usize;
    let num_aux = num_variables - num_inputs;
    let r1cs = circom_circuit::R1CS {
        num_aux,
        num_inputs,
        num_variables,
        constraints: r1cs_file.constraints,
    };

    let witness_data: &[u8] = load_bytes!("../circuits/Test2.wtns");
    let witness_byte_buf = ByteBuffer::from_bytes(witness_data);
    let witness = load_witness_from_u8_arr::<Bn256, BufReader<ByteBuffer>>(BufReader::new(witness_byte_buf)).ok();

    let circuit = circom_circuit::CircomCircuit {
        r1cs: r1cs,
        witness: witness,
        wire_mapping: None,
        aux_offset: plonk::AUX_OFFSET,
    };

    let srs_data: &[u8] = load_bytes!("../circuits/setup_2^20.key");
    let srs_byte_buf = ByteBuffer::from_bytes(srs_data);
    let srs_reader = BufReader::<ByteBuffer>::new(srs_byte_buf);
    let monomial_key = load_key_monomial_form_from_u8_arr::<Bn256>(srs_reader);

    let setup = plonk::SetupForProver::prepare_setup_for_prover(
        circuit.clone(),
        monomial_key,
        None
    )
    .expect("prepare err");

    log::info!("Proving...");
    let proof: Proof<Bn256, PlonkCsWidth4WithNextStepParams> = setup.prove(circuit).unwrap();

    let writer = File::create("proof.bin").unwrap();
    proof.write(writer).unwrap();

    println!("proof.opening_at_z_omega_proof: {}", proof.opening_at_z_omega_proof.to_string());

    let vk_data: &[u8] = load_bytes!("../circuits/Test2.vk");
    let vk_buf = ByteBuffer::from_bytes(vk_data);
    let mut vk_reader = BufReader::<ByteBuffer>::new(vk_buf);
    let vk = VerificationKey::<Bn256, PlonkCsWidth4WithNextStepParams>::read(&mut vk_reader).expect("read vk err");

    let correct = plonk::verify(&vk, &proof).unwrap();
    println!("{}", correct);
    if correct {
        println!("{}", "Proof is valid");
    } else {
        println!("{}", "Proof is invalid");
    }
}
