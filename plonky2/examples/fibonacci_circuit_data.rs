use std::fs;
use std::marker::PhantomData;
use anyhow::Result;
use dyn_clonable::dyn_clone::clone;
use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{CircuitConfig, VerifierCircuitData};
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use plonky2::util::serialization::{DefaultGateSerializer, DefaultGeneratorSerializer};

/// An example of using Plonky2 to prove a statement of the form
/// "I know the 100th element of the Fibonacci sequence, starting with constants a and b."
/// When a == 0 and b == 1, this is proving knowledge of the 100th (standard) Fibonacci number.
fn main() -> Result<()> {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);

    // The arithmetic circuit.
    let initial_a = builder.add_virtual_target();
    let initial_b = builder.add_virtual_target();
    let mut prev_target = initial_a;
    let mut cur_target = initial_b;
    for _ in 0..99 {
        let temp = builder.add(prev_target, cur_target);
        prev_target = cur_target;
        cur_target = temp;
    }

    // Public inputs are the two initial values (provided below) and the result (which is generated).
    builder.register_public_input(initial_a);
    builder.register_public_input(initial_b);
    builder.register_public_input(cur_target);

    // Provide initial values.
    let mut pw = PartialWitness::new();
    pw.set_target(initial_a, F::ZERO);
    pw.set_target(initial_b, F::ONE);

    let data = builder.build::<C>();
    let proof = data.prove(pw)?;
    let gate_serializer = DefaultGateSerializer;
    let verifier_data_bytes = data.verifier_data().to_bytes(&gate_serializer).expect("Error reading verifier data");
    let final_proof_path = "test.bin";
    fs::write(final_proof_path, &verifier_data_bytes).expect("Proof writing error");
    let verifier_data_bytes = fs::read(final_proof_path).expect("File not found");
    let verifier_circuit_data: VerifierCircuitData<F, C, D> = VerifierCircuitData::from_bytes(verifier_data_bytes, &gate_serializer).expect("Error serializing verifier circuit data");
    println!(
        "100th Fibonacci number mod |F| (starting with {}, {}) is: {}",
        proof.public_inputs[0], proof.public_inputs[1], proof.public_inputs[2]
    );
    verifier_circuit_data.verify(proof)
}