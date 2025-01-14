use plonky2::{
    field::extension::Extendable,
    hash::{
        hash_types::{HashOut, HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::{
        target::Target,
        witness::{PartialWitness, Witness},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{AlgebraicHasher, GenericConfig, Hasher},
        proof::{Proof, ProofWithPublicInputs},
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    poseidon::gadgets::poseidon_two_to_one,
    sparse_merkle_tree::{
        gadgets::process::process_smt::SmtProcessProof, goldilocks_poseidon::WrappedHashOut,
    },
    transaction::gadgets::{
        merge::{MergeProof, MergeTransitionTarget},
        purge::PurgeTransitionTarget,
    },
    zkdsa::account::Address,
};

// type C = PoseidonGoldilocksConfig;
// type H = <C as GenericConfig<D>>::InnerHasher;
// type F = <C as GenericConfig<D>>::F;
// const D: usize = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MergeAndPurgeTransition<F: RichField> {
    pub sender_address: Address<F>,
    pub merge_witnesses: Vec<MergeProof<F>>,
    pub purge_input_witnesses: Vec<(SmtProcessProof<F>, SmtProcessProof<F>, SmtProcessProof<F>)>,
    pub purge_output_witnesses: Vec<(SmtProcessProof<F>, SmtProcessProof<F>, SmtProcessProof<F>)>,
    pub nonce: WrappedHashOut<F>,
    pub old_user_asset_root: WrappedHashOut<F>,
}

pub struct MergeAndPurgeTransitionTarget<
    const N_LOG_MAX_USERS: usize,
    const N_LOG_MAX_TXS: usize,
    const N_LOG_MAX_CONTRACTS: usize,
    const N_LOG_MAX_VARIABLES: usize,
    const N_LOG_TXS: usize,
    const N_LOG_RECIPIENTS: usize,
    const N_LOG_CONTRACTS: usize,
    const N_LOG_VARIABLES: usize,
    const N_DIFFS: usize,
    const N_MERGES: usize,
    const N_DEPOSITS: usize,
> {
    pub merge_proof_target: MergeTransitionTarget<
        N_LOG_MAX_USERS,
        N_LOG_MAX_TXS,
        N_LOG_TXS,
        N_LOG_RECIPIENTS,
        N_MERGES,
        N_DEPOSITS,
    >,
    pub purge_proof_target: PurgeTransitionTarget<
        N_LOG_MAX_TXS,
        N_LOG_MAX_CONTRACTS,
        N_LOG_MAX_VARIABLES,
        N_LOG_RECIPIENTS,
        N_LOG_CONTRACTS,
        N_LOG_VARIABLES,
        N_DIFFS,
    >,
}

impl<
        const N_LOG_MAX_USERS: usize,
        const N_LOG_MAX_TXS: usize,
        const N_LOG_MAX_CONTRACTS: usize,
        const N_LOG_MAX_VARIABLES: usize,
        const N_LOG_TXS: usize,
        const N_LOG_RECIPIENTS: usize,
        const N_LOG_CONTRACTS: usize,
        const N_LOG_VARIABLES: usize,
        const N_DIFFS: usize,
        const N_MERGES: usize,
        const N_DEPOSITS: usize,
    >
    MergeAndPurgeTransitionTarget<
        N_LOG_MAX_USERS,
        N_LOG_MAX_TXS,
        N_LOG_MAX_CONTRACTS,
        N_LOG_MAX_VARIABLES,
        N_LOG_TXS,
        N_LOG_RECIPIENTS,
        N_LOG_CONTRACTS,
        N_LOG_VARIABLES,
        N_DIFFS,
        N_MERGES,
        N_DEPOSITS,
    >
{
    #[allow(clippy::too_many_arguments)]
    pub fn set_witness<F: RichField>(
        &self,
        pw: &mut impl Witness<F>,
        sender_address: Address<F>,
        merge_witnesses: &[MergeProof<F>],
        purge_input_witnesses: &[(SmtProcessProof<F>, SmtProcessProof<F>, SmtProcessProof<F>)],
        purge_output_witnesses: &[(SmtProcessProof<F>, SmtProcessProof<F>, SmtProcessProof<F>)],
        nonce: WrappedHashOut<F>,
        old_user_asset_root: WrappedHashOut<F>,
    ) -> MergeAndPurgeTransitionPublicInputs<F> {
        let middle_user_asset_root =
            self.merge_proof_target
                .set_witness(pw, merge_witnesses, *old_user_asset_root);
        let (new_user_asset_root, diff_root, tx_hash) = self.purge_proof_target.set_witness(
            pw,
            sender_address,
            purge_input_witnesses,
            purge_output_witnesses,
            middle_user_asset_root,
            nonce,
        );

        MergeAndPurgeTransitionPublicInputs {
            sender_address,
            old_user_asset_root,
            middle_user_asset_root,
            new_user_asset_root,
            diff_root,
            tx_hash,
        }
    }
}

pub fn make_user_proof_circuit<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
    const N_LOG_MAX_USERS: usize,
    const N_LOG_MAX_TXS: usize,
    const N_LOG_MAX_CONTRACTS: usize,
    const N_LOG_MAX_VARIABLES: usize,
    const N_LOG_TXS: usize,
    const N_LOG_RECIPIENTS: usize,
    const N_LOG_CONTRACTS: usize,
    const N_LOG_VARIABLES: usize,
    const N_DIFFS: usize,
    const N_MERGES: usize,
    const N_DEPOSITS: usize,
>(
    config: CircuitConfig,
    // zkdsa_circuit: SimpleSignatureCircuit,
) -> MergeAndPurgeTransitionCircuit<
    F,
    C,
    D,
    N_LOG_MAX_USERS,
    N_LOG_MAX_TXS,
    N_LOG_MAX_CONTRACTS,
    N_LOG_MAX_VARIABLES,
    N_LOG_TXS,
    N_LOG_RECIPIENTS,
    N_LOG_CONTRACTS,
    N_LOG_VARIABLES,
    N_DIFFS,
    N_MERGES,
    N_DEPOSITS,
>
where
    C::Hasher: AlgebraicHasher<F>,
{
    let mut builder = CircuitBuilder::<F, D>::new(config);
    // builder.debug_gate_row = Some(282);

    let merge_proof_target: MergeTransitionTarget<
        N_LOG_MAX_USERS,
        N_LOG_MAX_TXS,
        N_LOG_TXS,
        N_LOG_RECIPIENTS,
        N_MERGES,
        N_DEPOSITS,
    > = MergeTransitionTarget::add_virtual_to::<F, C::Hasher, D>(&mut builder);

    let purge_proof_target: PurgeTransitionTarget<
        N_LOG_MAX_TXS,
        N_LOG_MAX_CONTRACTS,
        N_LOG_MAX_VARIABLES,
        N_LOG_RECIPIENTS,
        N_LOG_CONTRACTS,
        N_LOG_VARIABLES,
        N_DIFFS,
    > = PurgeTransitionTarget::add_virtual_to::<F, C::Hasher, D>(&mut builder);
    builder.connect_hashes(
        merge_proof_target.new_user_asset_root,
        purge_proof_target.old_user_asset_root,
    );

    let tx_hash = poseidon_two_to_one::<F, C::Hasher, D>(
        &mut builder,
        purge_proof_target.diff_root,
        purge_proof_target.nonce,
    );

    // let public_inputs = MergeAndPurgeTransitionPublicInputsTarget {
    //     sender_address: purge_proof_target.sender_address.0,
    //     old_user_asset_root: merge_proof_target.old_user_asset_root,
    //     middle_user_asset_root: merge_proof_target.new_user_asset_root,
    //     new_user_asset_root: purge_proof_target.new_user_asset_root,
    //     diff_root: purge_proof_target.diff_root,
    //     tx_hash,
    // };
    // builder.register_public_inputs(&public_inputs.encode());
    builder.register_public_inputs(&merge_proof_target.old_user_asset_root.elements); // public_inputs[0..4]
    builder.register_public_inputs(&merge_proof_target.new_user_asset_root.elements); // public_inputs[4..8]
    builder.register_public_inputs(&purge_proof_target.new_user_asset_root.elements); // public_inputs[8..12]
    builder.register_public_inputs(&purge_proof_target.diff_root.elements); // public_inputs[12..16]
    builder.register_public_inputs(&purge_proof_target.sender_address.0.elements); // public_inputs[16..20]
    builder.register_public_inputs(&tx_hash.elements); // public_inputs[20..24]

    let targets = MergeAndPurgeTransitionTarget {
        // old_user_asset_root: merge_proof_target.old_user_asset_root,
        // new_user_asset_root: purge_proof_target.new_user_asset_root,
        merge_proof_target,
        purge_proof_target,
        // address: purge_proof_target.sender_address.clone(),
    };

    let merge_and_purge_circuit_data = builder.build::<C>();

    MergeAndPurgeTransitionCircuit {
        data: merge_and_purge_circuit_data,
        targets,
    }
}

pub struct MergeAndPurgeTransitionCircuit<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
    const N_LOG_MAX_USERS: usize,
    const N_LOG_MAX_TXS: usize,
    const N_LOG_MAX_CONTRACTS: usize,
    const N_LOG_MAX_VARIABLES: usize,
    const N_LOG_TXS: usize,
    const N_LOG_RECIPIENTS: usize,
    const N_LOG_CONTRACTS: usize,
    const N_LOG_VARIABLES: usize,
    const N_DIFFS: usize,
    const N_MERGES: usize,
    const N_DEPOSITS: usize,
> {
    pub data: CircuitData<F, C, D>,
    pub targets: MergeAndPurgeTransitionTarget<
        N_LOG_MAX_USERS,
        N_LOG_MAX_TXS,
        N_LOG_MAX_CONTRACTS,
        N_LOG_MAX_VARIABLES,
        N_LOG_TXS,
        N_LOG_RECIPIENTS,
        N_LOG_CONTRACTS,
        N_LOG_VARIABLES,
        N_DIFFS,
        N_MERGES,
        N_DEPOSITS,
    >,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(deserialize = "Address<F>: Deserialize<'de>, WrappedHashOut<F>: Deserialize<'de>"))]
pub struct MergeAndPurgeTransitionPublicInputs<F: RichField> {
    pub sender_address: Address<F>,
    pub old_user_asset_root: WrappedHashOut<F>,
    pub middle_user_asset_root: WrappedHashOut<F>,
    pub new_user_asset_root: WrappedHashOut<F>,
    pub diff_root: WrappedHashOut<F>,
    pub tx_hash: WrappedHashOut<F>,
}

impl<F: RichField> Default for MergeAndPurgeTransitionPublicInputs<F> {
    fn default() -> Self {
        let diff_root = Default::default();
        let nonce = Default::default();
        let tx_hash = PoseidonHash::two_to_one(diff_root, nonce);

        Self {
            sender_address: Default::default(),
            old_user_asset_root: Default::default(),
            middle_user_asset_root: Default::default(),
            new_user_asset_root: Default::default(),
            diff_root: diff_root.into(),
            tx_hash: tx_hash.into(),
        }
    }
}

#[test]
fn test_default_user_transaction() {
    use plonky2::field::{goldilocks_field::GoldilocksField, types::Field};

    type F = GoldilocksField;

    let default_user_transaction = MergeAndPurgeTransitionPublicInputs::<F>::default();

    let tx_hash = WrappedHashOut::from(HashOut {
        elements: [
            F::from_canonical_u64(4330397376401421145),
            F::from_canonical_u64(14124799381142128323),
            F::from_canonical_u64(8742572140681234676),
            F::from_canonical_u64(14345658006221440202),
        ],
    });

    assert_eq!(default_user_transaction.sender_address, Default::default());
    assert_eq!(
        default_user_transaction.old_user_asset_root,
        Default::default()
    );
    assert_eq!(
        default_user_transaction.middle_user_asset_root,
        Default::default()
    );
    assert_eq!(
        default_user_transaction.new_user_asset_root,
        Default::default()
    );
    assert_eq!(default_user_transaction.diff_root, Default::default());
    assert_eq!(default_user_transaction.tx_hash, tx_hash);
}

impl<F: RichField> MergeAndPurgeTransitionPublicInputs<F> {
    pub fn encode(&self) -> Vec<F> {
        let public_inputs = vec![
            self.old_user_asset_root.elements,
            self.middle_user_asset_root.elements,
            self.new_user_asset_root.elements,
            self.diff_root.elements,
            self.sender_address.elements,
            self.tx_hash.elements,
        ]
        .concat();
        assert_eq!(public_inputs.len(), 24);

        public_inputs
    }

    pub fn decode(public_inputs: &[F]) -> Self {
        assert_eq!(public_inputs.len(), 24);
        let old_user_asset_root = HashOut::from_partial(&public_inputs[0..4]).into();
        let middle_user_asset_root = HashOut::from_partial(&public_inputs[4..8]).into();
        let new_user_asset_root = HashOut::from_partial(&public_inputs[8..12]).into();
        let diff_root = HashOut::from_partial(&public_inputs[12..16]).into();
        let sender_address = Address(HashOut::from_partial(&public_inputs[16..20]));
        let tx_hash = HashOut::from_partial(&public_inputs[20..24]).into();

        Self {
            old_user_asset_root,
            middle_user_asset_root,
            new_user_asset_root,
            diff_root,
            sender_address,
            tx_hash,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MergeAndPurgeTransitionPublicInputsTarget {
    pub sender_address: HashOutTarget,
    pub old_user_asset_root: HashOutTarget,
    pub middle_user_asset_root: HashOutTarget,
    pub new_user_asset_root: HashOutTarget,
    pub diff_root: HashOutTarget,
    pub tx_hash: HashOutTarget,
}

impl MergeAndPurgeTransitionPublicInputsTarget {
    pub fn add_virtual_to<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        let sender_address = builder.add_virtual_hash();
        let old_user_asset_root = builder.add_virtual_hash();
        let middle_user_asset_root = builder.add_virtual_hash();
        let new_user_asset_root = builder.add_virtual_hash();
        let diff_root = builder.add_virtual_hash();
        let tx_hash = builder.add_virtual_hash();

        Self {
            sender_address,
            old_user_asset_root,
            middle_user_asset_root,
            new_user_asset_root,
            diff_root,
            tx_hash,
        }
    }

    pub fn set_witness<F: RichField>(
        &self,
        pw: &mut impl Witness<F>,
        public_inputs: &MergeAndPurgeTransitionPublicInputs<F>,
    ) {
        pw.set_hash_target(self.sender_address, *public_inputs.sender_address);
        pw.set_hash_target(self.old_user_asset_root, *public_inputs.old_user_asset_root);
        pw.set_hash_target(
            self.middle_user_asset_root,
            *public_inputs.middle_user_asset_root,
        );
        pw.set_hash_target(self.new_user_asset_root, *public_inputs.new_user_asset_root);
        pw.set_hash_target(self.diff_root, *public_inputs.diff_root);
        pw.set_hash_target(self.tx_hash, *public_inputs.tx_hash);
    }

    pub fn connect<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        a: &Self,
        b: &Self,
    ) {
        builder.connect_hashes(a.sender_address, b.sender_address);
        builder.connect_hashes(a.old_user_asset_root, b.old_user_asset_root);
        builder.connect_hashes(a.middle_user_asset_root, b.middle_user_asset_root);
        builder.connect_hashes(a.new_user_asset_root, b.new_user_asset_root);
        builder.connect_hashes(a.diff_root, b.diff_root);
        builder.connect_hashes(a.tx_hash, b.tx_hash);
    }

    pub fn encode(&self) -> Vec<Target> {
        let public_inputs_t = vec![
            self.old_user_asset_root.elements,
            self.middle_user_asset_root.elements,
            self.new_user_asset_root.elements,
            self.diff_root.elements,
            self.sender_address.elements,
            self.tx_hash.elements,
        ]
        .concat();
        assert_eq!(public_inputs_t.len(), 24);

        public_inputs_t
    }

    pub fn decode(public_inputs_t: &[Target]) -> Self {
        assert_eq!(public_inputs_t.len(), 24);
        let old_user_asset_root = HashOutTarget {
            elements: public_inputs_t[0..4].try_into().unwrap(),
        };
        let middle_user_asset_root = HashOutTarget {
            elements: public_inputs_t[4..8].try_into().unwrap(),
        };
        let new_user_asset_root = HashOutTarget {
            elements: public_inputs_t[8..12].try_into().unwrap(),
        };
        let diff_root = HashOutTarget {
            elements: public_inputs_t[12..16].try_into().unwrap(),
        };
        let sender_address = HashOutTarget {
            elements: public_inputs_t[16..20].try_into().unwrap(),
        };
        let tx_hash = HashOutTarget {
            elements: public_inputs_t[20..24].try_into().unwrap(),
        };

        MergeAndPurgeTransitionPublicInputsTarget {
            sender_address,
            old_user_asset_root,
            middle_user_asset_root,
            new_user_asset_root,
            diff_root,
            tx_hash,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct MergeAndPurgeTransitionProofWithPublicInputs<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
> {
    pub proof: Proof<F, C, D>,
    pub public_inputs: MergeAndPurgeTransitionPublicInputs<F>,
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>
    From<MergeAndPurgeTransitionProofWithPublicInputs<F, C, D>> for ProofWithPublicInputs<F, C, D>
{
    fn from(
        value: MergeAndPurgeTransitionProofWithPublicInputs<F, C, D>,
    ) -> ProofWithPublicInputs<F, C, D> {
        ProofWithPublicInputs {
            proof: value.proof,
            public_inputs: value.public_inputs.encode(),
        }
    }
}

// pub fn parse_merge_and_purge_public_inputs(
//     public_inputs_t: &[Target],
// ) -> MergeAndPurgeTransitionPublicInputsTarget {
//     let old_user_asset_root = HashOutTarget {
//         elements: public_inputs_t[0..4].try_into().unwrap(),
//     };
//     let middle_user_asset_root = HashOutTarget {
//         elements: public_inputs_t[4..8].try_into().unwrap(),
//     };
//     let new_user_asset_root = HashOutTarget {
//         elements: public_inputs_t[8..12].try_into().unwrap(),
//     };
//     let diff_root = HashOutTarget {
//         elements: public_inputs_t[12..16].try_into().unwrap(),
//     };
//     let sender_address = HashOutTarget {
//         elements: public_inputs_t[16..20].try_into().unwrap(),
//     };
//     let tx_hash = HashOutTarget {
//         elements: public_inputs_t[20..24].try_into().unwrap(),
//     };

//     MergeAndPurgeTransitionPublicInputsTarget {
//         sender_address,
//         old_user_asset_root,
//         middle_user_asset_root,
//         new_user_asset_root,
//         diff_root,
//         tx_hash,
//     }
// }

impl<
        F: RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
        const D: usize,
        const N_LOG_MAX_USERS: usize,
        const N_LOG_MAX_TXS: usize,
        const N_LOG_MAX_CONTRACTS: usize,
        const N_LOG_MAX_VARIABLES: usize,
        const N_LOG_TXS: usize,
        const N_LOG_RECIPIENTS: usize,
        const N_LOG_CONTRACTS: usize,
        const N_LOG_VARIABLES: usize,
        const N_DIFFS: usize,
        const N_MERGES: usize,
        const N_DEPOSITS: usize,
    >
    MergeAndPurgeTransitionCircuit<
        F,
        C,
        D,
        N_LOG_MAX_USERS,
        N_LOG_MAX_TXS,
        N_LOG_MAX_CONTRACTS,
        N_LOG_MAX_VARIABLES,
        N_LOG_TXS,
        N_LOG_RECIPIENTS,
        N_LOG_CONTRACTS,
        N_LOG_VARIABLES,
        N_DIFFS,
        N_MERGES,
        N_DEPOSITS,
    >
{
    pub fn parse_public_inputs(&self) -> MergeAndPurgeTransitionPublicInputsTarget {
        let public_inputs_t = self.data.prover_only.public_inputs.clone();

        MergeAndPurgeTransitionPublicInputsTarget::decode(&public_inputs_t)
    }

    pub fn prove(
        &self,
        inputs: PartialWitness<F>,
    ) -> anyhow::Result<MergeAndPurgeTransitionProofWithPublicInputs<F, C, D>> {
        let proof_with_pis = self.data.prove(inputs)?;
        let public_inputs =
            MergeAndPurgeTransitionPublicInputs::decode(&proof_with_pis.public_inputs);

        Ok(MergeAndPurgeTransitionProofWithPublicInputs {
            proof: proof_with_pis.proof,
            public_inputs,
        })
    }

    pub fn set_witness_and_prove(
        &self,
        sender_address: Address<F>,
        merge_witnesses: &[MergeProof<F>],
        purge_input_witnesses: &[(SmtProcessProof<F>, SmtProcessProof<F>, SmtProcessProof<F>)],
        purge_output_witnesses: &[(SmtProcessProof<F>, SmtProcessProof<F>, SmtProcessProof<F>)],
        nonce: WrappedHashOut<F>,
        old_user_asset_root: WrappedHashOut<F>,
    ) -> anyhow::Result<MergeAndPurgeTransitionProofWithPublicInputs<F, C, D>> {
        let mut pw = PartialWitness::new();
        self.targets.set_witness(
            &mut pw,
            sender_address,
            merge_witnesses,
            purge_input_witnesses,
            purge_output_witnesses,
            nonce,
            old_user_asset_root,
        );

        self.prove(pw)
    }

    pub fn verify(
        &self,
        proof_with_pis: MergeAndPurgeTransitionProofWithPublicInputs<F, C, D>,
    ) -> anyhow::Result<()> {
        self.data
            .verify(ProofWithPublicInputs::from(proof_with_pis))
    }
}

/// witness を入力にとり、 user_tx_proof を返す関数
pub fn prove_user_transaction<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
    const N_LOG_MAX_USERS: usize,
    const N_LOG_MAX_TXS: usize,
    const N_LOG_MAX_CONTRACTS: usize,
    const N_LOG_MAX_VARIABLES: usize,
    const N_LOG_TXS: usize,
    const N_LOG_RECIPIENTS: usize,
    const N_LOG_CONTRACTS: usize,
    const N_LOG_VARIABLES: usize,
    const N_DIFFS: usize,
    const N_MERGES: usize,
    const N_DEPOSITS: usize,
>(
    sender_address: Address<F>,
    merge_witnesses: &[MergeProof<F>],
    purge_input_witnesses: &[(SmtProcessProof<F>, SmtProcessProof<F>, SmtProcessProof<F>)],
    purge_output_witnesses: &[(SmtProcessProof<F>, SmtProcessProof<F>, SmtProcessProof<F>)],
    nonce: WrappedHashOut<F>,
    old_user_asset_root: WrappedHashOut<F>,
) -> anyhow::Result<MergeAndPurgeTransitionProofWithPublicInputs<F, C, D>>
where
    C::Hasher: AlgebraicHasher<F>,
{
    // let config = CircuitConfig::standard_recursion_zk_config(); // TODO
    let config = CircuitConfig::standard_recursion_config();
    let merge_and_purge_circuit = make_user_proof_circuit::<
        F,
        C,
        D,
        N_LOG_MAX_USERS,
        N_LOG_MAX_TXS,
        N_LOG_MAX_CONTRACTS,
        N_LOG_MAX_VARIABLES,
        N_LOG_TXS,
        N_LOG_RECIPIENTS,
        N_LOG_CONTRACTS,
        N_LOG_VARIABLES,
        N_DIFFS,
        N_MERGES,
        N_DEPOSITS,
    >(config);

    let mut pw = PartialWitness::new();
    let _public_inputs = merge_and_purge_circuit.targets.set_witness(
        &mut pw,
        sender_address,
        merge_witnesses,
        purge_input_witnesses,
        purge_output_witnesses,
        nonce,
        old_user_asset_root,
    );

    let user_tx_proof = merge_and_purge_circuit
        .prove(pw)
        .map_err(|err| anyhow::anyhow!("fail to prove user transaction: {}", err))?;

    Ok(user_tx_proof)
}
