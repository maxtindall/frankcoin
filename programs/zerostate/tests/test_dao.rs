//! End-to-end: mine a real frankcoin proof, then join / propose / vote, and
//! prove the gates hold — no proof means no citizen, no citizen means no vote,
//! and no citizen can vote twice.
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
const RENT: Pubkey = Pubkey::from_str_const("SysvarRent111111111111111111111111111111111");

fn send(svm: &mut LiteSVM, ixs: &[Instruction], payer: &Keypair, signers: &[&Keypair]) -> bool {
    let bh = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &bh);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx).is_ok()
}

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

fn ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref(), TOKEN.as_ref(), mint.as_ref()], &ATA_PROG).0
}

/// Register + mine one frankcoin proof for `who`. Returns nothing; leaves the
/// wallet with a Proof account showing count >= 1.
fn mine_once(svm: &mut LiteSVM, fc: &Pubkey, mint: &Pubkey, config: &Pubkey, who: &Keypair) {
    let proof = Pubkey::find_program_address(&[b"proof", who.pubkey().as_ref()], fc).0;
    let reg = Instruction {
        program_id: *fc,
        accounts: frankcoin::accounts::Register {
            miner: who.pubkey(), config: *config, proof,
            system_program: Pubkey::default(),
        }.to_account_metas(None),
        data: frankcoin::instruction::Register {}.data(),
    };
    assert!(send(svm, &[reg], who, &[who]), "register failed");

    let challenge = frankcoin::state::Proof::try_deserialize(
        &mut svm.get_account(&proof).unwrap().data.as_slice()).unwrap().challenge;
    let nonce = grind(&challenge, &who.pubkey(), 8);
    let mine = Instruction {
        program_id: *fc,
        accounts: frankcoin::accounts::Mine {
            miner: who.pubkey(), config: *config, mint: *mint, proof,
            miner_ata: ata(&who.pubkey(), mint),
            token_program: TOKEN, associated_token_program: ATA_PROG,
            system_program: Pubkey::default(),
        }.to_account_metas(None),
        data: frankcoin::instruction::Mine { nonce }.data(),
    };
    assert!(send(svm, &[mine], who, &[who]), "mine failed");
}

#[test]
fn one_miner_one_vote() {
    let fc = frankcoin::id();
    let dao_id = zerostate::id();
    let mut svm = LiteSVM::new();
    svm.add_program(fc, include_bytes!("../../../target/deploy/frankcoin.so")).unwrap();
    svm.add_program(dao_id, include_bytes!("../../../target/deploy/zerostate.so")).unwrap();

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 1_000_000_000_000).unwrap();

    // frankcoin genesis
    let config = Pubkey::find_program_address(&[b"config"], &fc).0;
    let mint = Pubkey::find_program_address(&[b"mint"], &fc).0;
    let init_fc = Instruction {
        program_id: fc,
        accounts: frankcoin::accounts::Initialize {
            payer: payer.pubkey(), config, mint, token_program: TOKEN,
            system_program: Pubkey::default(), rent: RENT,
        }.to_account_metas(None),
        data: frankcoin::instruction::Initialize { difficulty: 8, cooldown: 0 }.data(),
    };
    assert!(send(&mut svm, &[init_fc], &payer, &[&payer]), "frankcoin init failed");

    // found the DAO
    let dao = Pubkey::find_program_address(&[b"dao"], &dao_id).0;
    let init_dao = Instruction {
        program_id: dao_id,
        accounts: zerostate::accounts::Initialize {
            founder: payer.pubkey(), dao, system_program: Pubkey::default(),
        }.to_account_metas(None),
        data: zerostate::instruction::Initialize {}.data(),
    };
    assert!(send(&mut svm, &[init_dao], &payer, &[&payer]), "dao init failed");

    // --- two miners, and one wallet that never mined ---
    let alice = Keypair::new();
    let bob = Keypair::new();
    let freeloader = Keypair::new();
    for k in [&alice, &bob, &freeloader] {
        svm.airdrop(&k.pubkey(), 1_000_000_000_000).unwrap();
    }
    mine_once(&mut svm, &fc, &mint, &config, &alice);
    mine_once(&mut svm, &fc, &mint, &config, &bob);

    let citizen = |who: &Pubkey| Pubkey::find_program_address(&[b"citizen", who.as_ref()], &dao_id).0;
    let fc_proof = |who: &Pubkey| Pubkey::find_program_address(&[b"proof", who.as_ref()], &fc).0;
    // `authority` admits `member`; mining is verified against member's proof.
    let admit_ix = |authority: &Keypair, member: &Pubkey| Instruction {
        program_id: dao_id,
        accounts: zerostate::accounts::Admit {
            admit_authority: authority.pubkey(), dao, member: *member,
            proof: fc_proof(member), citizen: citizen(member),
            system_program: Pubkey::default(),
        }.to_account_metas(None),
        data: zerostate::instruction::Admit {}.data(),
    };

    // payer is the founder = admit authority. It admits the two miners.
    assert!(send(&mut svm, &[admit_ix(&payer, &alice.pubkey())], &payer, &[&payer]),
        "authority should admit alice");
    assert!(send(&mut svm, &[admit_ix(&payer, &bob.pubkey())], &payer, &[&payer]),
        "authority should admit bob");

    // the freeloader never mined -> no Proof account -> cannot be admitted,
    // even by the authority. Mining is a hard floor.
    assert!(!send(&mut svm, &[admit_ix(&payer, &freeloader.pubkey())], &payer, &[&payer]),
        "a wallet that never mined cannot be admitted");

    // a non-authority cannot admit, even a legitimate miner. Entry is trusted.
    let carol = Keypair::new();
    svm.airdrop(&carol.pubkey(), 1_000_000_000_000).unwrap();
    mine_once(&mut svm, &fc, &mint, &config, &carol);
    assert!(!send(&mut svm, &[admit_ix(&alice, &carol.pubkey())], &alice, &[&alice]),
        "a citizen who is not the authority cannot admit anyone");

    // alice proposes
    let proposal = Pubkey::find_program_address(&[b"proposal", &0u64.to_le_bytes()], &dao_id).0;
    let propose = Instruction {
        program_id: dao_id,
        accounts: zerostate::accounts::Propose {
            proposer: alice.pubkey(), dao, citizen: citizen(&alice.pubkey()),
            proposal, system_program: Pubkey::default(),
        }.to_account_metas(None),
        data: zerostate::instruction::Propose {
            title: "should 0state acquire its first parcel of land?".to_string(),
            body_hash: [7u8; 32],
        }.data(),
    };
    assert!(send(&mut svm, &[propose], &alice, &[&alice]), "alice should propose");

    let ballot = |voter: &Pubkey| Pubkey::find_program_address(
        &[b"ballot", proposal.as_ref(), citizen(voter).as_ref()], &dao_id).0;
    let vote_ix = |voter: &Keypair, choice: u8| Instruction {
        program_id: dao_id,
        accounts: zerostate::accounts::Vote {
            voter: voter.pubkey(), citizen: citizen(&voter.pubkey()),
            proposal, ballot: ballot(&voter.pubkey()), system_program: Pubkey::default(),
        }.to_account_metas(None),
        data: zerostate::instruction::Vote { choice }.data(),
    };

    // both vote yes
    assert!(send(&mut svm, &[vote_ix(&alice, 1)], &alice, &[&alice]), "alice votes");
    assert!(send(&mut svm, &[vote_ix(&bob, 1)], &bob, &[&bob]), "bob votes");

    // alice cannot vote twice — the ballot PDA already exists
    assert!(!send(&mut svm, &[vote_ix(&alice, 0)], &alice, &[&alice]),
        "a citizen must not vote twice");

    // the freeloader is not a citizen, so has no citizen account -> cannot vote
    assert!(!send(&mut svm, &[vote_ix(&freeloader, 1)], &freeloader, &[&freeloader]),
        "a non-citizen must not vote");

    // tally: two citizens, two yes, and every citizen counted for exactly one
    let p = zerostate::state::Proposal::try_deserialize(
        &mut svm.get_account(&proposal).unwrap().data.as_slice()).unwrap();
    assert_eq!(p.yes, 2, "two equal yes votes");
    assert_eq!(p.no, 0);
    assert_eq!(p.abstain, 0);

    let d = zerostate::state::Dao::try_deserialize(
        &mut svm.get_account(&dao).unwrap().data.as_slice()).unwrap();
    assert_eq!(d.citizen_count, 2, "exactly the two admitted miners are citizens");
    assert_eq!(d.proposal_count, 1);
    assert_eq!(d.admit_authority, payer.pubkey(), "the founder holds the door");

    // --- revoke: trust is withdrawable, by the authority only ---
    let revoke_ix = |authority: &Keypair, member: &Pubkey| Instruction {
        program_id: dao_id,
        accounts: zerostate::accounts::Revoke {
            admit_authority: authority.pubkey(), dao, member: *member,
            citizen: citizen(member),
        }.to_account_metas(None),
        data: zerostate::instruction::Revoke {}.data(),
    };
    // a non-authority cannot revoke
    assert!(!send(&mut svm, &[revoke_ix(&bob, &alice.pubkey())], &bob, &[&bob]),
        "a citizen cannot revoke another");
    // the authority can
    assert!(send(&mut svm, &[revoke_ix(&payer, &bob.pubkey())], &payer, &[&payer]),
        "the authority may revoke bob");
    let d2 = zerostate::state::Dao::try_deserialize(
        &mut svm.get_account(&dao).unwrap().data.as_slice()).unwrap();
    assert_eq!(d2.citizen_count, 1, "bob is no longer a citizen");
    assert!(svm.get_account(&citizen(&bob.pubkey())).map_or(true, |a| a.data.is_empty()),
        "bob's citizen account is closed");
}
