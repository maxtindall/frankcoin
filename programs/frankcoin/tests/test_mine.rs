use anchor_lang::{
    solana_program::instruction::Instruction, AccountDeserialize, InstructionData, ToAccountMetas,
};
use litesvm::LiteSVM;
use solana_keccak_hasher::hashv;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

const TOKEN: Pubkey = anchor_spl::token::ID;
const ATA_PROG: Pubkey = anchor_spl::associated_token::ID;

fn ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref(), TOKEN.as_ref(), mint.as_ref()], &ATA_PROG).0
}

fn send(svm: &mut LiteSVM, ixs: &[Instruction], payer: &Keypair, signers: &[&Keypair]) -> bool {
    let bh = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &bh);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx).is_ok()
}

/// Grind a nonce so keccak(challenge || miner || nonce) has >= difficulty
/// leading zero bits — the same computation the program verifies.
fn grind(challenge: &[u8; 32], miner: &Pubkey, difficulty: u32) -> u64 {
    for nonce in 0u64.. {
        let h = hashv(&[challenge, miner.as_ref(), &nonce.to_le_bytes()]).to_bytes();
        let mut zeros = 0u32;
        for b in h.iter() {
            if *b == 0 { zeros += 8; } else { zeros += b.leading_zeros(); break; }
        }
        if zeros >= difficulty { return nonce; }
    }
    unreachable!()
}

fn token_balance(svm: &LiteSVM, ata: &Pubkey) -> u64 {
    let acc = svm.get_account(ata).expect("ata exists");
    // SPL token account: amount is u64 LE at offset 64.
    u64::from_le_bytes(acc.data[64..72].try_into().unwrap())
}

#[test]
fn mine_grants_reward_and_enforces_rules() {
    let program_id = frankcoin::id();
    let mut svm = LiteSVM::new();
    svm.add_program(program_id, include_bytes!("../../../target/deploy/frankcoin.so"))
        .unwrap();

    let payer = Keypair::new();
    let miner = Keypair::new();
    svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();
    svm.airdrop(&miner.pubkey(), 100_000_000_000).unwrap();

    let config = Pubkey::find_program_address(&[b"config"], &program_id).0;
    let mint = Pubkey::find_program_address(&[b"mint"], &program_id).0;
    let proof = Pubkey::find_program_address(&[b"proof", miner.pubkey().as_ref()], &program_id).0;

    // --- initialize: low difficulty (8 bits) and no cooldown for a fast test ---
    let init = Instruction {
        program_id,
        accounts: frankcoin::accounts::Initialize {
            payer: payer.pubkey(),
            config,
            mint,
            token_program: TOKEN,
            system_program: solana_pubkey::Pubkey::default(),
            rent: Pubkey::from_str_const("SysvarRent111111111111111111111111111111111"),
        }
        .to_account_metas(None),
        data: frankcoin::instruction::Initialize { difficulty: 8, cooldown: 0 }.data(),
    };
    assert!(send(&mut svm, &[init], &payer, &[&payer]), "initialize failed");

    // --- register the miner ---
    let reg = Instruction {
        program_id,
        accounts: frankcoin::accounts::Register {
            miner: miner.pubkey(),
            config,
            proof,
            system_program: solana_pubkey::Pubkey::default(),
        }
        .to_account_metas(None),
        data: frankcoin::instruction::Register {}.data(),
    };
    assert!(send(&mut svm, &[reg], &miner, &[&miner]), "register failed");

    let miner_ata = ata(&miner.pubkey(), &mint);

    // --- mine once: grind a valid nonce against the current challenge ---
    let read_challenge = |svm: &LiteSVM| -> [u8; 32] {
        let data = svm.get_account(&proof).unwrap().data;
        frankcoin::state::Proof::try_deserialize(&mut data.as_slice())
            .unwrap()
            .challenge
    };

    let mk_mine = |nonce: u64| Instruction {
        program_id,
        accounts: frankcoin::accounts::Mine {
            miner: miner.pubkey(),
            config,
            mint,
            proof,
            miner_ata,
            token_program: TOKEN,
            associated_token_program: ATA_PROG,
            system_program: solana_pubkey::Pubkey::default(),
        }
        .to_account_metas(None),
        data: frankcoin::instruction::Mine { nonce }.data(),
    };

    let n1 = grind(&read_challenge(&svm), &miner.pubkey(), 8);
    assert!(send(&mut svm, &[mk_mine(n1)], &miner, &[&miner]), "first mine failed");
    assert_eq!(token_balance(&svm, &miner_ata), 500_000_000_000, "first reward should be 500 FRANKS");

    // --- config.total_minted tracks it ---
    let cfg = frankcoin::state::Config::try_deserialize(
        &mut svm.get_account(&config).unwrap().data.as_slice(),
    )
    .unwrap();
    assert_eq!(cfg.total_minted, 500_000_000_000);
    assert_eq!(cfg.proofs_accepted, 1);

    // --- an old/invalid nonce must be rejected (challenge has rolled) ---
    assert!(!send(&mut svm, &[mk_mine(n1)], &miner, &[&miner]),
        "stale nonce must be rejected after the challenge rolls");

    // --- mine again with a freshly ground nonce -> 100 FRANKS ---
    let n2 = grind(&read_challenge(&svm), &miner.pubkey(), 8);
    assert!(send(&mut svm, &[mk_mine(n2)], &miner, &[&miner]), "second mine failed");
    assert_eq!(token_balance(&svm, &miner_ata), 1_000_000_000_000, "two rewards = 1000 FRANKS");
}
