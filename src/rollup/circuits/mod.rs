use plonky2::{
    field::extension::Extendable,
    hash::{
        hash_types::{HashOut, HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::{
        target::{BoolTarget, Target},
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
    merkle_tree::{
        gadgets::{get_merkle_root_target_from_leaves, MerkleProofTarget},
        tree::{get_merkle_proof, get_merkle_root},
    },
    recursion::gadgets::RecursiveProofTarget,
    rollup::gadgets::{
        approval_block::ApprovalBlockProductionTarget,
        block_headers_tree::calc_block_headers_proof,
        deposit_block::{
            DepositBlockProductionTarget, DepositInfo, DepositInfoTarget, VariableIndex,
        },
        proposal_block::ProposalBlockProductionTarget,
    },
    sparse_merkle_tree::{
        gadgets::process::process_smt::SmtProcessProof, goldilocks_poseidon::WrappedHashOut,
    },
    transaction::{
        block_header::{get_block_hash, BlockHeader},
        circuits::{
            make_user_proof_circuit, MergeAndPurgeTransition, MergeAndPurgeTransitionCircuit,
            MergeAndPurgeTransitionProofWithPublicInputs,
            MergeAndPurgeTransitionPublicInputsTarget,
        },
        gadgets::block_header::{get_block_hash_target, BlockHeaderTarget},
    },
    zkdsa::{
        account::Address,
        circuits::{
            make_simple_signature_circuit, SimpleSignatureCircuit,
            SimpleSignatureProofWithPublicInputs, SimpleSignaturePublicInputsTarget,
        },
        gadgets::account::AddressTarget,
    },
};

use super::{
    address_list::TransactionSenderWithValidity,
    gadgets::address_list::TransactionSenderWithValidityTarget,
};

// type C = PoseidonGoldilocksConfig;
// type H = <C as GenericConfig<D>>::InnerHasher;
// type F = <C as GenericConfig<D>>::F;
// const D: usize = 2;
const N_LOG_MAX_BLOCKS: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "F: RichField + Extendable<D>, C: GenericConfig<D, F = F>")]
pub struct BlockDetail<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
    pub block_number: u32,
    pub user_tx_proofs: Vec<MergeAndPurgeTransitionProofWithPublicInputs<F, C, D>>,
    pub deposit_process_proofs: Vec<(SmtProcessProof<F>, SmtProcessProof<F>, SmtProcessProof<F>)>,
    pub world_state_process_proofs: Vec<SmtProcessProof<F>>,
    pub world_state_revert_proofs: Vec<SmtProcessProof<F>>,
    pub received_signature_proofs: Vec<Option<SimpleSignatureProofWithPublicInputs<F, C, D>>>,
    pub latest_account_process_proofs: Vec<SmtProcessProof<F>>,
    pub block_headers_proof_siblings: Vec<WrappedHashOut<F>>,
    pub prev_block_header: BlockHeader<F>,
}

// impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> Default
//     for BlockDetail<F, C, D>
// {
//     fn default() -> Self {
//         unimplemented!("please use `new` function instead")
//     }
// }

#[test]
fn test_serde_block_detail() {
    use plonky2::plonk::config::PoseidonGoldilocksConfig;

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    const N_TXS: usize = 4;

    let block_detail: BlockDetail<F, C, D> = BlockDetail::new(N_TXS);
    let encoded_block_detail = "{\"block_number\":1,\"user_tx_proofs\":[],\"deposit_process_proofs\":[],\"world_state_process_proofs\":[],\"world_state_revert_proofs\":[],\"received_signature_proofs\":[],\"latest_account_process_proofs\":[],\"block_headers_proof_siblings\":[\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"0xc71603f33a1144ca7953db0ab48808f4c4055e3364a246c33c18a9786cb0b359\",\"0x2196fc41328ae503de8f9ad762a30af28d85581b9901b2cfb61a4ad1aaf14fcc\",\"0x67703a0cc73ca54246fb94bfe956c05f9a247cc59da2de6461e00af7295ce05a\",\"0xf522eaa0af88a040167d7cf3bf854d278cc1b30d2e2c09475154921a06462644\",\"0xd0053597686f6672b77e23f0fc59019786ac9b34bd97d439e9e6b5c8d15b61ae\",\"0x49561260080d30c3dda8f741c47dfb105a1d2a648eee8f0325225f1a5d49614a\",\"0xb768e4fc8b0b79f516c9da6ea83aa4b13c9a42c646c4c1f9e979ed3ee20855e3\",\"0x2bd367124a2989b3d31bd45195f9a9278d72cff3db0a7a5afe6fd7720cfd2916\",\"0xfcf1da35791ff4452cf0c633ee9d9197954ec02c35af849e3ca2442157c9f14e\",\"0xc27e8f4600af2a41707c71f51d338df791e919b1e4a3ea53ccf7b63f7b1140c3\",\"0x218bc75b3bc83675e1c5ac76b0d9d44c0d1baab6f05098e38d6ebaad0ab5d3c3\",\"0x61618c69e9d26f4c8ee39e4c215804e2fb01846fee718016ed2589168e839d21\",\"0xec76a20799cf5dc50841b1fa4588f4f8c975d7aec7a1c669296ff821d8378f7f\",\"0xf55d5d12107b371efb4650fb6b8880811f7867621b8c1c1a0168a392cc7b542c\",\"0x6c9890682b94dee9cd45643c378df78c64e3f7a7160f8f0de73c5360c4b3ecd8\",\"0x9e1c5239e937026b57b8f931187d6dc4b555892ea200cfe4ab95f0ae94f7cde6\",\"0x0aa45be01f9e161002f8e22c79467775279949e14530c2505587ad00b6ddf0cb\",\"0xd2e3dd2bdd2907959ef35a5aeb905682388540a0f77810a8d108cd9026164f3b\",\"0x33a8e0b809ce2532ae94d561f2e16def904fa2e7b99bd3f1707d95a1148000a1\",\"0x7c9f51793bca6ffb713d0a918edaa60557184cbbc85f535743926baabe5db81f\",\"0xfa58391e7c0d394d317903270df6e518b34770c62a38e6697621f88cdcdfb5fd\",\"0x60e99b7ea5b1187d4293a24d51cc07ac39f874beb115877f8bd1878dd7f1026d\",\"0xc043477d124292017879345b4f881eb71d31cd8564acce2a617f3c6d0b4b8b44\",\"0x5793fc6d609c47c365b9470bc3e00cd4f19dece13278be693612ac9d812a8f8c\",\"0xe0c55886db8e5a00bfa58f8faf71ab1e1f12ae8ff82875c95b3c0f2c8ee070cc\",\"0x8f3c07c1b1e0b6c9c69aade405671398bf062e3f77dc0b13671c5e28b2f9dc9a\",\"0x06ff527899c10074411162bf4a7f70b84e6acab68322cba1e9e10aca93469e78\",\"0x08b8d7b96221d9f59ed49f4906c24becbe646c8d1b68665bf42d09eff74e4b90\",\"0xe0fd1bfa878b3cd2cc7e2bf5f351da7a2a1963d1913370406b4ae756e5e20763\",\"0x80faf1e491cd910ae2566bc52d26d7ea099b512bfeff20768a0dd4cf966a4a93\",\"0x20ca8d0d3b8c55d18b0f02df1c469ca317afad6c010c855f7765a145976afdbc\"],\"prev_block_header\":{\"block_number\":\"0x00000000\",\"prev_block_hash\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"block_headers_digest\":\"0xd65af5933a094e8329332a714327ba72b1e4dac93c0cde8ee479b9bb36c3fc43\",\"transactions_digest\":\"0xd0053597686f6672b77e23f0fc59019786ac9b34bd97d439e9e6b5c8d15b61ae\",\"deposit_digest\":\"0xf522eaa0af88a040167d7cf3bf854d278cc1b30d2e2c09475154921a06462644\",\"proposed_world_state_digest\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"approved_world_state_digest\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"latest_account_digest\":\"0x0000000000000000000000000000000000000000000000000000000000000000\"}}";
    let decoded_block_detail: BlockDetail<F, C, D> =
        serde_json::from_str(encoded_block_detail).unwrap();
    assert_eq!(decoded_block_detail, block_detail);
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>
    BlockDetail<F, C, D>
{
    pub fn new(log_num_txs_in_block: usize) -> Self {
        let prev_block_header = BlockHeader::new(log_num_txs_in_block);
        let prev_block_hash = get_block_hash(&prev_block_header);
        let prev_block_number = prev_block_header.block_number;
        let mut block_headers: Vec<WrappedHashOut<F>> =
            vec![WrappedHashOut::ZERO; prev_block_number as usize];
        block_headers.push(prev_block_hash.into());
        let block_number = prev_block_number + 1;
        let user_tx_proofs = vec![];
        let received_signature_proofs = vec![];
        let world_state_process_proofs = vec![];
        let world_state_revert_proofs = vec![];
        let latest_account_process_proofs = vec![];
        let block_headers_proof_siblings =
            get_merkle_proof(&block_headers, prev_block_number as usize, N_LOG_MAX_BLOCKS).siblings;

        let deposit_process_proofs = vec![];

        Self {
            block_number,
            user_tx_proofs,
            deposit_process_proofs,
            world_state_process_proofs,
            world_state_revert_proofs,
            received_signature_proofs,
            latest_account_process_proofs,
            block_headers_proof_siblings,
            prev_block_header,
        }
    }
}

#[derive(Clone)]
pub struct BlockProductionTarget<
    const D: usize,
    const N_LOG_USERS: usize, // N_LOG_MAX_USERS
    const N_LOG_TXS: usize,
    const N_LOG_RECIPIENTS: usize,
    const N_LOG_CONTRACTS: usize,
    const N_LOG_VARIABLES: usize,
    const N_TXS: usize,
    const N_DEPOSITS: usize,
> {
    pub deposit_block_target: DepositBlockProductionTarget<
        D,
        N_LOG_RECIPIENTS,
        N_LOG_CONTRACTS,
        N_LOG_VARIABLES,
        N_DEPOSITS,
    >,
    pub proposal_block_target: ProposalBlockProductionTarget<D, N_LOG_USERS, N_TXS>,
    pub approval_block_target: ApprovalBlockProductionTarget<D, N_LOG_USERS, N_TXS>,
    pub user_tx_proofs: [RecursiveProofTarget<D>; N_TXS],
    pub received_signature_proofs: [RecursiveProofTarget<D>; N_TXS],
    pub block_headers_proof: MerkleProofTarget<N_LOG_MAX_BLOCKS>,
    pub prev_block_header: BlockHeaderTarget,
    pub block_header: BlockHeaderTarget,
}

impl<
        const D: usize,
        const N_LOG_USERS: usize,
        const N_LOG_TXS: usize,
        const N_LOG_RECIPIENTS: usize,
        const N_LOG_CONTRACTS: usize,
        const N_LOG_VARIABLES: usize,
        const N_TXS: usize,
        const N_DEPOSITS: usize,
    >
    BlockProductionTarget<
        D,
        N_LOG_USERS,
        N_LOG_TXS,
        N_LOG_RECIPIENTS,
        N_LOG_CONTRACTS,
        N_LOG_VARIABLES,
        N_TXS,
        N_DEPOSITS,
    >
{
    /// Returns `(block_header, address_list)`.
    #[allow(clippy::too_many_arguments)]
    pub fn set_witness<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>>(
        &self,
        pw: &mut impl Witness<F>,
        block_number: u32,
        user_tx_proofs: &[MergeAndPurgeTransitionProofWithPublicInputs<F, C, D>],
        default_user_tx_proof: &MergeAndPurgeTransitionProofWithPublicInputs<F, C, D>,
        deposit_process_proofs: &[(SmtProcessProof<F>, SmtProcessProof<F>, SmtProcessProof<F>)],
        world_state_process_proofs: &[SmtProcessProof<F>],
        world_state_revert_proofs: &[SmtProcessProof<F>],
        received_signature_proofs: &[Option<SimpleSignatureProofWithPublicInputs<F, C, D>>],
        default_simple_signature_proof: &SimpleSignatureProofWithPublicInputs<F, C, D>,
        latest_account_process_proofs: &[SmtProcessProof<F>],
        block_headers_proof_siblings: &[WrappedHashOut<F>],
        prev_block_header: BlockHeader<F>,
    ) -> BlockProductionPublicInputs<F, N_TXS, N_DEPOSITS>
    where
        C::Hasher: AlgebraicHasher<F>,
    {
        let interior_deposit_digest = self
            .deposit_block_target
            .set_witness(pw, deposit_process_proofs);
        let old_world_state_root = prev_block_header.approved_world_state_digest.into();
        let user_transactions = user_tx_proofs
            .iter()
            .cloned()
            .map(|p| p.public_inputs)
            .collect::<Vec<_>>();
        let (transactions_digest, proposed_world_state_digest) =
            self.proposal_block_target.set_witness(
                pw,
                world_state_process_proofs,
                &user_transactions,
                old_world_state_root,
            );
        let old_latest_account_root = prev_block_header.latest_account_digest.into();
        let received_signatures = received_signature_proofs
            .iter()
            .cloned()
            .map(|p| p.map(|p| p.public_inputs))
            .collect::<Vec<_>>();
        let (approved_world_state_digest, latest_account_digest) =
            self.approval_block_target.set_witness(
                pw,
                block_number,
                world_state_revert_proofs,
                &user_transactions,
                &received_signatures,
                latest_account_process_proofs,
                proposed_world_state_digest,
                old_latest_account_root,
            );

        assert!(user_tx_proofs.len() <= self.user_tx_proofs.len());
        for (r_t, r) in self.user_tx_proofs.iter().zip(user_tx_proofs.iter()) {
            r_t.set_witness(pw, &ProofWithPublicInputs::from(r.clone()), true);
        }

        for r_t in self.user_tx_proofs.iter().skip(user_tx_proofs.len()) {
            r_t.set_witness(
                pw,
                &ProofWithPublicInputs::from(default_user_tx_proof.clone()),
                false,
            );
        }

        assert!(received_signature_proofs.len() <= self.received_signature_proofs.len());
        for (r_t, r) in self
            .received_signature_proofs
            .iter()
            .zip(received_signature_proofs.iter())
        {
            let r: Option<&_> = r.into();
            r_t.set_witness(
                pw,
                &ProofWithPublicInputs::from(r.unwrap_or(default_simple_signature_proof).clone()),
                r.is_some(),
            );
        }

        for r_t in self
            .received_signature_proofs
            .iter()
            .skip(received_signature_proofs.len())
        {
            r_t.set_witness(
                pw,
                &ProofWithPublicInputs::from(default_simple_signature_proof.clone()),
                false,
            );
        }

        self.prev_block_header.set_witness(pw, &prev_block_header);
        for (sibling_t, sibling) in self
            .block_headers_proof
            .siblings
            .iter()
            .zip(block_headers_proof_siblings.iter().cloned())
        {
            pw.set_hash_target(*sibling_t, *sibling);
        }

        let prev_block_number = prev_block_header.block_number;

        // `block_number - 2` までの block header で作られた block headers tree の `block_number - 1` 番目の proof
        // この時点では, leaf の値は 0 である.
        let prev_block_headers_digest = get_merkle_root(
            prev_block_number as usize,
            WrappedHashOut::ZERO,
            block_headers_proof_siblings,
        );
        assert_eq!(
            *prev_block_headers_digest,
            prev_block_header.block_headers_digest,
        );
        // `block_number - 1` の block hash
        let prev_block_hash = get_block_hash(&prev_block_header);
        // `block_number - 1` までの block header で作られた block headers tree の `block_number - 1` 番目の proof
        let block_headers_digest = get_merkle_root(
            prev_block_number as usize,
            prev_block_hash.into(),
            block_headers_proof_siblings,
        );

        let deposit_digest = get_merkle_proof(&[interior_deposit_digest], 0, N_LOG_TXS).root;

        let block_header = BlockHeader {
            block_number,
            prev_block_hash,
            transactions_digest: *transactions_digest,
            deposit_digest: *deposit_digest,
            proposed_world_state_digest: *proposed_world_state_digest,
            approved_world_state_digest: *approved_world_state_digest,
            latest_account_digest: *latest_account_digest,
            block_headers_digest: *block_headers_digest,
        };

        let block_hash = get_block_hash(&block_header);

        let mut address_list = user_transactions
            .iter()
            .zip(received_signatures.iter())
            .map(
                |(user_tx_proof, received_signature_proof)| TransactionSenderWithValidity {
                    sender_address: user_tx_proof.sender_address,
                    is_valid: received_signature_proof.is_some(),
                },
            )
            .collect::<Vec<_>>();
        address_list.resize(N_TXS, TransactionSenderWithValidity::default());

        let mut deposit_list = deposit_process_proofs
            .iter()
            .map(|proof_t| DepositInfo {
                receiver_address: Address(*proof_t.0.new_key),
                contract_address: Address(*proof_t.1.new_key),
                variable_index: VariableIndex::from_hash_out(*proof_t.2.new_key),
                amount: proof_t.2.new_value.elements[0],
            })
            .collect::<Vec<_>>();
        deposit_list.resize(N_DEPOSITS, DepositInfo::default());

        BlockProductionPublicInputs {
            address_list: address_list.try_into().unwrap(),
            deposit_list: deposit_list.try_into().unwrap(),
            old_account_tree_root: prev_block_header.latest_account_digest,
            new_account_tree_root: block_header.latest_account_digest,
            old_world_state_root: prev_block_header.approved_world_state_digest,
            new_world_state_root: block_header.approved_world_state_digest,
            old_prev_block_header_digest: prev_block_header.block_headers_digest,
            new_prev_block_header_digest: block_header.block_headers_digest,
            block_hash,
        }
    }
}

pub fn make_block_proof_circuit<
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
    const N_TXS: usize,
    const N_DEPOSITS: usize,
>(
    config: CircuitConfig,
    merge_and_purge_circuit: &MergeAndPurgeTransitionCircuit<
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
    >,
    simple_signature_circuit: &SimpleSignatureCircuit<F, C, D>,
) -> BlockProductionCircuit<
    F,
    C,
    D,
    N_LOG_MAX_USERS,
    N_LOG_TXS,
    N_LOG_RECIPIENTS,
    N_LOG_CONTRACTS,
    N_LOG_VARIABLES,
    N_TXS,
    N_DEPOSITS,
>
where
    C::Hasher: AlgebraicHasher<F>,
{
    let mut builder = CircuitBuilder::<F, D>::new(config);

    // deposit block
    let deposit_block_target: DepositBlockProductionTarget<
        D,
        N_LOG_RECIPIENTS,
        N_LOG_CONTRACTS,
        N_LOG_VARIABLES,
        N_DEPOSITS,
    > = DepositBlockProductionTarget::add_virtual_to::<F, <C as GenericConfig<D>>::Hasher>(
        &mut builder,
    );

    // proposal block
    let proposal_block_target: ProposalBlockProductionTarget<D, N_LOG_MAX_USERS, N_TXS> =
        ProposalBlockProductionTarget::add_virtual_to::<F, <C as GenericConfig<D>>::Hasher>(
            &mut builder,
        );

    // approval block
    let approval_block_target: ApprovalBlockProductionTarget<D, N_LOG_MAX_USERS, N_TXS> =
        ApprovalBlockProductionTarget::add_virtual_to::<F, <C as GenericConfig<D>>::Hasher>(
            &mut builder,
        );

    let user_tx_proofs = [0; N_TXS]
        .map(|_| RecursiveProofTarget::add_virtual_to(&mut builder, &merge_and_purge_circuit.data));

    let mut transaction_hashes_t = vec![];
    for ((u, p), a) in user_tx_proofs
        .iter()
        .zip(proposal_block_target.user_transactions.iter())
        .zip(approval_block_target.user_transactions.iter())
    {
        let user_tx_public_inputs =
            MergeAndPurgeTransitionPublicInputsTarget::decode(&u.inner.0.public_inputs);
        MergeAndPurgeTransitionPublicInputsTarget::connect(&mut builder, p, &user_tx_public_inputs);
        MergeAndPurgeTransitionPublicInputsTarget::connect(&mut builder, a, &user_tx_public_inputs);
        transaction_hashes_t.push(user_tx_public_inputs.tx_hash);
    }

    let received_signature_proofs = [0; N_TXS].map(|_| {
        RecursiveProofTarget::add_virtual_to(&mut builder, &simple_signature_circuit.data)
    });

    for (r, a) in received_signature_proofs
        .iter()
        .zip(approval_block_target.received_signatures.iter())
    {
        let signature = SimpleSignaturePublicInputsTarget::decode(&r.inner.0.public_inputs);
        SimpleSignaturePublicInputsTarget::connect(&mut builder, &a.0, &signature);
    }

    let address_list = proposal_block_target
        .user_transactions
        .iter()
        .zip(approval_block_target.received_signatures.iter())
        .map(
            |(user_tx_proof, received_signature)| TransactionSenderWithValidityTarget {
                sender_address: user_tx_proof.sender_address,
                is_valid: received_signature.1,
            },
        )
        .collect::<Vec<_>>();

    let deposit_list = deposit_block_target
        .deposit_process_proofs
        .iter()
        .map(|proof_t| DepositInfoTarget {
            receiver_address: AddressTarget(proof_t.0.new_key),
            contract_address: AddressTarget(proof_t.1.new_key),
            variable_index: proof_t.2.new_key,
            amount: proof_t.2.new_value.elements[0],
        })
        .collect::<Vec<_>>();

    // block header
    let block_number = approval_block_target.current_block_number;
    builder.range_check(block_number, N_LOG_MAX_BLOCKS);
    let one = builder.one();
    let prev_block_number = builder.sub(block_number, one);
    builder.range_check(prev_block_number, N_LOG_MAX_BLOCKS);

    let transactions_digest = proposal_block_target.transactions_digest;
    let interior_deposit_digest = deposit_block_target.interior_deposit_digest;
    let prev_world_state_digest = proposal_block_target.old_world_state_root;
    let proposed_world_state_digest = proposal_block_target.new_world_state_root;
    let approved_world_state_digest = approval_block_target.new_world_state_root;
    let prev_latest_account_digest = approval_block_target.old_latest_account_root;
    let latest_account_digest = approval_block_target.new_latest_account_root;

    let prev_block_header = BlockHeaderTarget {
        block_number: prev_block_number,
        block_headers_digest: builder.add_virtual_hash(),
        transactions_digest: builder.add_virtual_hash(),
        deposit_digest: builder.add_virtual_hash(),
        proposed_world_state_digest: builder.add_virtual_hash(),
        approved_world_state_digest: prev_world_state_digest,
        latest_account_digest: prev_latest_account_digest,
    };

    let prev_block_headers_proof_siblings =
        [0; N_LOG_MAX_BLOCKS].map(|_| builder.add_virtual_hash());
    let prev_block_headers_digest = prev_block_header.block_headers_digest;
    let block_headers_proof = calc_block_headers_proof::<F, C::Hasher, D>(
        &mut builder,
        prev_block_headers_proof_siblings,
        &prev_block_header,
    );

    let zero = builder.zero();
    let default_hash = HashOutTarget::from_partial(&[], zero);
    let deposit_digest = {
        let mut deposit_tree_leaves = vec![interior_deposit_digest];
        deposit_tree_leaves.resize(N_TXS, default_hash);

        get_merkle_root_target_from_leaves::<F, C::Hasher, D>(&mut builder, deposit_tree_leaves)
    };

    let block_header = BlockHeaderTarget {
        block_number,
        block_headers_digest: block_headers_proof.root,
        transactions_digest,
        deposit_digest,
        proposed_world_state_digest,
        approved_world_state_digest,
        latest_account_digest,
    };
    let block_hash = get_block_hash_target::<F, C::Hasher, D>(&mut builder, &block_header);

    let public_inputs: BlockProductionPublicInputsTarget<N_TXS, N_DEPOSITS> =
        BlockProductionPublicInputsTarget {
            address_list: address_list.try_into().unwrap(),
            deposit_list: deposit_list.try_into().unwrap(),
            old_account_tree_root: approval_block_target.old_latest_account_root,
            new_account_tree_root: approval_block_target.new_latest_account_root,
            old_world_state_root: proposal_block_target.old_world_state_root,
            new_world_state_root: approval_block_target.new_world_state_root,
            old_block_headers_root: prev_block_headers_digest,
            new_block_headers_root: block_headers_proof.root,
            block_hash,
        };
    let entry_hash = public_inputs.get_entry_hash::<F, C::Hasher, D>(&mut builder);
    builder.register_public_inputs(&entry_hash.elements);
    let block_circuit_data = builder.build::<C>();

    let targets = BlockProductionTarget {
        proposal_block_target,
        approval_block_target,
        deposit_block_target,
        user_tx_proofs,
        received_signature_proofs,
        block_headers_proof,
        prev_block_header,
        block_header,
    };

    BlockProductionCircuit {
        data: block_circuit_data,
        targets,
    }
}

pub struct BlockProductionCircuit<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
    const N_LOG_USERS: usize,
    const N_LOG_TXS: usize,
    const N_LOG_RECIPIENTS: usize,
    const N_LOG_CONTRACTS: usize,
    const N_LOG_VARIABLES: usize,
    const N_TXS: usize,
    const N_DEPOSITS: usize,
> {
    pub data: CircuitData<F, C, D>,
    pub targets: BlockProductionTarget<
        D,
        N_LOG_USERS,
        N_LOG_TXS,
        N_LOG_RECIPIENTS,
        N_LOG_CONTRACTS,
        N_LOG_VARIABLES,
        N_TXS,
        N_DEPOSITS,
    >,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockProductionPublicInputs<F: RichField, const N_TXS: usize, const N_DEPOSITS: usize> {
    pub address_list: [TransactionSenderWithValidity<F>; N_TXS],
    pub deposit_list: [DepositInfo<F>; N_DEPOSITS],
    pub old_account_tree_root: HashOut<F>,
    pub new_account_tree_root: HashOut<F>,
    pub old_world_state_root: HashOut<F>,
    pub new_world_state_root: HashOut<F>,
    pub old_prev_block_header_digest: HashOut<F>,
    pub new_prev_block_header_digest: HashOut<F>,
    pub block_hash: HashOut<F>,
}

impl<F: RichField, const N_TXS: usize, const N_DEPOSITS: usize>
    BlockProductionPublicInputs<F, N_TXS, N_DEPOSITS>
{
    pub fn encode(&self) -> Vec<F> {
        let mut public_inputs = vec![];
        for TransactionSenderWithValidity {
            sender_address,
            is_valid,
        } in self.address_list
        {
            sender_address.write(&mut public_inputs);
            // public_inputs.push(last_block_number);
            public_inputs.push(F::from_bool(is_valid));
        }

        for _ in (0..N_TXS).skip(self.address_list.len()) {
            Address::default().write(&mut public_inputs);
            // public_inputs.push(last_block_number);
            public_inputs.push(F::from_bool(false));
        }

        for DepositInfo {
            receiver_address,
            contract_address,
            variable_index,
            amount,
        } in self.deposit_list
        {
            receiver_address.write(&mut public_inputs);
            contract_address.write(&mut public_inputs);
            variable_index.write(&mut public_inputs);
            public_inputs.push(amount);
        }

        for _ in (0..N_DEPOSITS).skip(self.deposit_list.len()) {
            Address::default().write(&mut public_inputs);
            Address::default().write(&mut public_inputs);
            VariableIndex::from(0u8).write(&mut public_inputs);
            public_inputs.push(F::ZERO);
        }

        WrappedHashOut::from(self.old_account_tree_root).write(&mut public_inputs);
        WrappedHashOut::from(self.new_account_tree_root).write(&mut public_inputs);
        WrappedHashOut::from(self.old_world_state_root).write(&mut public_inputs);
        WrappedHashOut::from(self.new_world_state_root).write(&mut public_inputs);

        WrappedHashOut::from(self.old_prev_block_header_digest).write(&mut public_inputs);
        WrappedHashOut::from(self.new_prev_block_header_digest).write(&mut public_inputs);
        WrappedHashOut::from(self.block_hash).write(&mut public_inputs);

        public_inputs
    }

    pub fn decode(public_inputs: &[F]) -> Self {
        assert_eq!(public_inputs.len(), 5 * N_TXS + 13 * N_DEPOSITS + 28);

        let mut public_inputs = public_inputs.iter();

        let address_list = [(); N_TXS].map(|_| TransactionSenderWithValidity {
            sender_address: Address::read(&mut public_inputs),
            is_valid: public_inputs.next().unwrap().is_nonzero(),
        });
        let deposit_list = [(); N_DEPOSITS].map(|_| DepositInfo {
            receiver_address: Address::read(&mut public_inputs),
            contract_address: Address::read(&mut public_inputs),
            variable_index: VariableIndex::read(&mut public_inputs),
            amount: *public_inputs.next().unwrap(),
        });
        let old_account_tree_root = *WrappedHashOut::read(&mut public_inputs);
        let new_account_tree_root = *WrappedHashOut::read(&mut public_inputs);

        let old_world_state_root = *WrappedHashOut::read(&mut public_inputs);
        let new_world_state_root = *WrappedHashOut::read(&mut public_inputs);
        let old_prev_block_header_digest = *WrappedHashOut::read(&mut public_inputs);
        let new_prev_block_header_digest = *WrappedHashOut::read(&mut public_inputs);
        let block_hash = *WrappedHashOut::read(&mut public_inputs);

        assert_eq!(public_inputs.next(), None);

        Self {
            address_list,
            deposit_list,
            old_account_tree_root,
            new_account_tree_root,
            old_world_state_root,
            new_world_state_root,
            old_prev_block_header_digest,
            new_prev_block_header_digest,
            block_hash,
        }
    }

    pub fn get_entry_hash(&self) -> HashOut<F> {
        PoseidonHash::hash_no_pad(&self.encode())
    }
}

#[derive(Clone, Debug)]
pub struct BlockProductionPublicInputsTarget<const N_TXS: usize, const N_DEPOSITS: usize> {
    pub address_list: [TransactionSenderWithValidityTarget; N_TXS],
    pub deposit_list: [DepositInfoTarget; N_DEPOSITS],
    pub old_account_tree_root: HashOutTarget,
    pub new_account_tree_root: HashOutTarget,
    pub old_world_state_root: HashOutTarget,
    pub new_world_state_root: HashOutTarget,
    pub old_block_headers_root: HashOutTarget,
    pub new_block_headers_root: HashOutTarget,
    pub block_hash: HashOutTarget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockProductionProofWithPublicInputs<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
    const N_TXS: usize,
    const N_DEPOSITS: usize,
> {
    pub proof: Proof<F, C, D>,
    pub public_inputs: BlockProductionPublicInputs<F, N_TXS, N_DEPOSITS>,
}

impl<
        F: RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
        const D: usize,
        const N_TXS: usize,
        const N_DEPOSITS: usize,
    > From<BlockProductionProofWithPublicInputs<F, C, D, N_TXS, N_DEPOSITS>>
    for ProofWithPublicInputs<F, C, D>
{
    fn from(
        value: BlockProductionProofWithPublicInputs<F, C, D, N_TXS, N_DEPOSITS>,
    ) -> ProofWithPublicInputs<F, C, D> {
        let entry_hash = value.public_inputs.get_entry_hash();

        ProofWithPublicInputs {
            proof: value.proof,
            public_inputs: entry_hash.elements.to_vec(),
        }
    }
}

impl<const N_TXS: usize, const N_DEPOSITS: usize>
    BlockProductionPublicInputsTarget<N_TXS, N_DEPOSITS>
{
    pub fn encode(&self) -> Vec<Target> {
        let flatten_address_list_t = self
            .address_list
            .iter()
            .flat_map(|v| {
                vec![v.sender_address.elements.to_vec(), vec![v.is_valid.target]].concat()
            })
            .collect::<Vec<Target>>();
        let flatten_deposit_list_t = self
            .deposit_list
            .iter()
            .flat_map(|v| {
                vec![
                    v.receiver_address.0.elements.to_vec(),
                    v.contract_address.0.elements.to_vec(),
                    v.variable_index.elements.to_vec(),
                    vec![v.amount],
                ]
                .concat()
            })
            .collect::<Vec<Target>>();
        let public_inputs_t = vec![
            flatten_address_list_t,
            flatten_deposit_list_t,
            self.old_account_tree_root.elements.to_vec(),
            self.new_account_tree_root.elements.to_vec(),
            self.old_world_state_root.elements.to_vec(),
            self.new_world_state_root.elements.to_vec(),
            self.old_block_headers_root.elements.to_vec(),
            self.new_block_headers_root.elements.to_vec(),
            self.block_hash.elements.to_vec(),
        ]
        .concat();

        assert_eq!(public_inputs_t.len(), 5 * N_TXS + 13 * N_DEPOSITS + 28);

        public_inputs_t
    }

    pub fn decode(public_inputs_t: &[Target]) -> Self {
        assert_eq!(public_inputs_t.len(), 5 * N_TXS + 13 * N_DEPOSITS + 28);

        let mut public_inputs_t = public_inputs_t.iter();
        let address_list = (0..N_TXS)
            .map(|_| TransactionSenderWithValidityTarget {
                sender_address: HashOutTarget {
                    elements: [
                        *public_inputs_t.next().unwrap(),
                        *public_inputs_t.next().unwrap(),
                        *public_inputs_t.next().unwrap(),
                        *public_inputs_t.next().unwrap(),
                    ],
                },
                // last_block_number: *public_inputs_t.next().unwrap(),
                is_valid: BoolTarget::new_unsafe(*public_inputs_t.next().unwrap()),
            })
            .collect::<Vec<_>>();

        let deposit_list = (0..N_DEPOSITS)
            .map(|_| DepositInfoTarget {
                receiver_address: AddressTarget::read(&mut public_inputs_t),
                contract_address: AddressTarget::read(&mut public_inputs_t),
                variable_index: HashOutTarget {
                    elements: [
                        *public_inputs_t.next().unwrap(),
                        *public_inputs_t.next().unwrap(),
                        *public_inputs_t.next().unwrap(),
                        *public_inputs_t.next().unwrap(),
                    ],
                },
                amount: *public_inputs_t.next().unwrap(),
            })
            .collect::<Vec<_>>();

        let old_account_tree_root = HashOutTarget {
            elements: [
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
            ],
        };
        let new_account_tree_root = HashOutTarget {
            elements: [
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
            ],
        };

        let old_world_state_root = HashOutTarget {
            elements: [
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
            ],
        };
        let new_world_state_root = HashOutTarget {
            elements: [
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
            ],
        };
        let old_block_headers_root = HashOutTarget {
            elements: [
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
            ],
        };
        let new_block_headers_root = HashOutTarget {
            elements: [
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
            ],
        };
        let block_hash = HashOutTarget {
            elements: [
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
                *public_inputs_t.next().unwrap(),
            ],
        };

        assert_eq!(public_inputs_t.next(), None);

        BlockProductionPublicInputsTarget {
            address_list: address_list.try_into().unwrap(),
            deposit_list: deposit_list.try_into().unwrap(),
            old_account_tree_root,
            new_account_tree_root,
            old_world_state_root,
            new_world_state_root,
            old_block_headers_root,
            new_block_headers_root,
            block_hash,
        }
    }

    pub fn get_entry_hash<F: RichField + Extendable<D>, H: AlgebraicHasher<F>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> HashOutTarget {
        builder.hash_n_to_hash_no_pad::<H>(self.encode())
    }
}

#[test]
fn test_encode_block_production_public_inputs() {
    use plonky2::{field::types::Sample, plonk::config::PoseidonGoldilocksConfig};
    use rand::random;

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    const N_TXS: usize = 4;
    const N_DEPOSITS: usize = 2;

    let public_inputs = BlockProductionPublicInputs {
        address_list: [(); N_TXS].map(|_| TransactionSenderWithValidity {
            sender_address: Address::rand(),
            is_valid: random(),
        }),
        deposit_list: [(); N_DEPOSITS].map(|_| DepositInfo {
            receiver_address: Address::rand(),
            contract_address: Address::rand(),
            variable_index: VariableIndex::from(random::<u8>()),
            amount: F::rand(),
        }),
        old_account_tree_root: *WrappedHashOut::rand(),
        new_account_tree_root: *WrappedHashOut::rand(),
        old_world_state_root: *WrappedHashOut::rand(),
        new_world_state_root: *WrappedHashOut::rand(),
        old_prev_block_header_digest: *WrappedHashOut::rand(),
        new_prev_block_header_digest: *WrappedHashOut::rand(),
        block_hash: *WrappedHashOut::rand(),
    };
    let encoded_public_inputs = public_inputs.encode();
    let decoded_public_inputs = BlockProductionPublicInputs::decode(&encoded_public_inputs);
    assert_eq!(decoded_public_inputs, public_inputs, "invalid entry hash");
}

impl<
        F: RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
        const D: usize,
        const N_LOG_USERS: usize,
        const N_LOG_TXS: usize,
        const N_LOG_RECIPIENTS: usize,
        const N_LOG_CONTRACTS: usize,
        const N_LOG_VARIABLES: usize,
        const N_TXS: usize,
        const N_DEPOSITS: usize,
    >
    BlockProductionCircuit<
        F,
        C,
        D,
        N_LOG_USERS,
        N_LOG_TXS,
        N_LOG_RECIPIENTS,
        N_LOG_CONTRACTS,
        N_LOG_VARIABLES,
        N_TXS,
        N_DEPOSITS,
    >
where
    C::Hasher: AlgebraicHasher<F>,
{
    pub fn parse_public_inputs(&self) -> BlockProductionPublicInputsTarget<N_TXS, N_DEPOSITS> {
        let public_inputs_t = self.data.prover_only.public_inputs.clone();

        BlockProductionPublicInputsTarget::decode(&public_inputs_t)
    }

    // pub fn prove(
    //     &self,
    //     _inputs: PartialWitness<F>,
    // ) -> anyhow::Result<BlockProductionProofWithPublicInputs<F, C, D>> {
    //     unimplemented!("use set_witness_and_prove instead");
    // }

    pub fn set_witness_and_prove(
        &self,
        input: &BlockDetail<F, C, D>,
        default_user_tx_proof: &MergeAndPurgeTransitionProofWithPublicInputs<F, C, D>,
        default_simple_signature_proof: &SimpleSignatureProofWithPublicInputs<F, C, D>,
    ) -> anyhow::Result<BlockProductionProofWithPublicInputs<F, C, D, N_TXS, N_DEPOSITS>> {
        let mut pw = PartialWitness::new();
        let public_inputs = self.targets.set_witness::<F, C>(
            &mut pw,
            input.block_number,
            &input.user_tx_proofs,
            default_user_tx_proof,
            &input.deposit_process_proofs,
            &input.world_state_process_proofs,
            &input.world_state_revert_proofs,
            &input.received_signature_proofs,
            default_simple_signature_proof,
            &input.latest_account_process_proofs,
            &input.block_headers_proof_siblings,
            input.prev_block_header.clone(),
        );

        let proof_with_pis = self.data.prove(pw)?;
        if proof_with_pis.public_inputs.len() != 4 {
            anyhow::bail!("invalid length of public inputs");
        }
        let entry_hash = HashOut::from_partial(&proof_with_pis.public_inputs[..4]);
        if entry_hash != public_inputs.get_entry_hash() {
            anyhow::bail!("invalid entry hash");
        }

        Ok(BlockProductionProofWithPublicInputs {
            proof: proof_with_pis.proof,
            public_inputs,
        })
    }

    pub fn verify(
        &self,
        proof_with_pis: BlockProductionProofWithPublicInputs<F, C, D, N_TXS, N_DEPOSITS>,
    ) -> anyhow::Result<()> {
        self.data
            .verify(ProofWithPublicInputs::from(proof_with_pis))
    }
}

/// witness を入力にとり、 block_production_proof を返す関数
pub fn prove_block_production<
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
    const N_TXS: usize,
    const N_DEPOSITS: usize,
>(
    input: &BlockDetail<F, C, D>,
) -> anyhow::Result<BlockProductionProofWithPublicInputs<F, C, D, N_TXS, N_DEPOSITS>>
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
    let default_user_transaction = MergeAndPurgeTransition::default();
    let default_user_tx_proof = merge_and_purge_circuit
        .set_witness_and_prove(
            default_user_transaction.sender_address,
            &default_user_transaction.merge_witnesses,
            &default_user_transaction.purge_input_witnesses,
            &default_user_transaction.purge_output_witnesses,
            default_user_transaction.nonce,
            default_user_transaction.old_user_asset_root,
        )
        .map_err(|err| anyhow::anyhow!("fail to prove user transaction: {}", err))?;

    // let config = CircuitConfig::standard_recursion_zk_config(); // TODO
    let config = CircuitConfig::standard_recursion_config();
    let simple_signature_circuit = make_simple_signature_circuit::<F, C, D>(config);
    let default_simple_signature_proof = simple_signature_circuit
        .set_witness_and_prove(Default::default(), Default::default())
        .map_err(|err| anyhow::anyhow!("fail to prove simple signature: {}", err))?;

    let config = CircuitConfig::standard_recursion_config();
    let block_production_circuit =
        make_block_proof_circuit::<
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
            N_TXS,
            N_DEPOSITS,
        >(config, &merge_and_purge_circuit, &simple_signature_circuit);

    let block_production_proof = block_production_circuit
        .set_witness_and_prove(
            input,
            &default_user_tx_proof,
            &default_simple_signature_proof,
        )
        .map_err(|err| anyhow::anyhow!("fail to prove block production: {}", err))?;

    block_production_circuit
        .verify(block_production_proof.clone())
        .map_err(|err| anyhow::anyhow!("fail to verify block production proof: {}", err))?;

    Ok(block_production_proof)
}

#[test]
fn test_prove_block_production() {
    use plonky2::plonk::config::PoseidonGoldilocksConfig;

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    const N_LOG_MAX_USERS: usize = 3;
    const N_LOG_MAX_TXS: usize = 3;
    const N_LOG_MAX_CONTRACTS: usize = 3;
    const N_LOG_MAX_VARIABLES: usize = 3;
    const N_LOG_TXS: usize = 2;
    const N_LOG_RECIPIENTS: usize = 3;
    const N_LOG_CONTRACTS: usize = 3;
    const N_LOG_VARIABLES: usize = 3;
    const N_DIFFS: usize = 2;
    const N_MERGES: usize = 2;
    const N_TXS: usize = 2usize.pow(N_LOG_TXS as u32);
    const N_DEPOSITS: usize = 2;

    let default_block_details: BlockDetail<F, C, D> = BlockDetail::new(N_TXS);
    let _default_block_production_proof = prove_block_production::<
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
        N_TXS,
        N_DEPOSITS,
    >(&default_block_details)
    .unwrap();
}
