use std::{path::PathBuf, str::FromStr};
use litesvm::LiteSVM;
use litesvm_token::{CreateMint, spl_token};
use sha2::{Digest, Sha256};
use solana_sdk::{
    declare_id, message::{AccountMeta, Instruction}, pubkey::Pubkey, signature::{Keypair, Signer, read_keypair_file}, transaction::Transaction
};
use borsh::BorshDeserialize;


//************************* DECLARATIONS *************************//

const POOL_SEED: &str = "POOL";
declare_id!("11111111111111111111111111111111");

#[derive(Debug, BorshDeserialize)]
pub struct Pool {
    pub admin: Pubkey,
    pub stake_mint: Pubkey,
    pub reward_mint: Pubkey,
    pub stake_vault: Pubkey,
    pub reward_vault: Pubkey,
    pub reward_rate: u64,
    pub total_stake: u128,
    pub total_shares: u128,
    pub acc_reward_per_share: u128,
    pub last_update_time: i64,
    pub paused: bool,
    pub bump: u8,
}

//************************* HELPER FUNCTIONS *************************//

fn program_keypair_path() -> PathBuf {
    // Start at the crate root (programs/staking_smartcontract)
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Go up to workspace root, then into target/deploy
    path.push("..");
    path.push("..");
    path.push("target/deploy/staking_smartcontract-keypair.json");
    path
}

// Helper function to derive Pool PDA
fn get_pool_pda(stake_mint: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
       &[POOL_SEED.as_bytes(), stake_mint.as_ref()],
        program_id,
    )
}

// Helper function to derive stake or reward vault PDA
fn get_stake_or_reward_vault_pda(keyword: &str, pool: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
       &[keyword.as_bytes(), pool.as_ref()],
        program_id,
    )
}

// Helper function to calculate instruction discriminator
fn get_discriminator(instruction_name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("global:{}", instruction_name));
    let result = hasher.finalize();
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&result[..8]);
    discriminator
}

// Helper to create a token mint
fn create_token_mint(svm: &mut LiteSVM, payer: &Keypair) -> Pubkey {
    let mint = CreateMint::new(svm, &payer)
    .authority(&payer.pubkey())
    .decimals(9)
    .send()
    .unwrap();

    mint
}


//************************* TEST CASES *************************//

#[test]
fn program_deployment() {
    let mut svm = LiteSVM::new();

    let program_keypair = read_keypair_file(program_keypair_path()).expect("Program keypair file not found");
    let program_id = program_keypair.pubkey();

    let program_bytes = include_bytes!("../../../target/deploy/staking_smartcontract.so");

    svm.add_program(program_id, program_bytes).expect("Failed to deploy programs");

    assert!(svm.get_account(&program_id).is_some(), "Program account not created");
    assert!(svm.get_account(&program_id).unwrap().executable, "Program not executable");
}

#[test]
fn initialize_staking() {
    // Initialize the test environment
    let mut svm = LiteSVM::new();
    // Create an Admin
    let admin = Keypair::new();
    
    // Drop some lamports to the admin
    svm.airdrop(&admin.pubkey(), 1_000_000_000).unwrap();

    // Get the staking program keypair
    let program_keypair = read_keypair_file(program_keypair_path()).expect("Program keypair file not found");
    // Get the address of the staking program
    let program_id = program_keypair.pubkey();
    let program_bytes = include_bytes!("../../../target/deploy/staking_smartcontract.so");

    // Deploy the staking program
    svm.add_program(program_id, program_bytes).expect("Failed to deploy programs");
    // Create token min
    let mint = create_token_mint(&mut svm, &admin);

    // Derive the pool pda
    let (pool_pda, bump) = get_pool_pda(&mint, &program_id);
    // Derive the stake and reward vault pda
    let (stake_vault_pda, _bump) = get_stake_or_reward_vault_pda("stake_vault", &pool_pda, &program_id);
    let (reward_vault_pda, _bump) = get_stake_or_reward_vault_pda("reward_vault", &pool_pda, &program_id);

    const REWARD_RATE: u64 = 115_740;

    let discriminator = get_discriminator("initialize_pool");
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&discriminator);
    let reward_rate_bytes = REWARD_RATE.to_ne_bytes();
    instruction_data.extend_from_slice(&reward_rate_bytes);

    // Build the instruction to initialize staking pool
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(pool_pda, false),
            AccountMeta::new(mint, false),
            AccountMeta::new(mint, false),
            AccountMeta::new(stake_vault_pda, false),
            AccountMeta::new(reward_vault_pda, false),
            AccountMeta::new(spl_token::id(), false),
            AccountMeta::new(id(), false),
        ],
        data: instruction_data,
    };

    // Create and sign the transaction
    let tx = Transaction::new_signed_with_payer(
       &[instruction], 
        Some(&admin.pubkey()), 
        &[&admin], 
        svm.latest_blockhash()
    );

    match svm.send_transaction(tx) {
        Ok(meta) => {
            println!("Initialize pool transaction successful: {}", meta.signature);
            println!("Compute units: {}", meta.compute_units_consumed);
        }
        Err(err) => {
            println!("Transaction failed: {:?}", err.err);
            println!("Failure logs: {:?}", err.meta.logs);
        }
    }

    let pool_account = svm.get_account(&pool_pda).expect("Pool account should exist");
    let pool = Pool::deserialize(&mut &pool_account.data[8..]).expect("Failed to deserialize Pool");

    assert_eq!(pool.admin, admin.pubkey());
    assert_eq!(pool.stake_mint, mint);
    assert_eq!(pool.reward_mint, mint);
    assert_eq!(pool.stake_vault, stake_vault_pda);
    assert_eq!(pool.reward_vault, reward_vault_pda);
    assert_eq!(pool.reward_rate, REWARD_RATE);
    assert_eq!(pool.total_stake, 0);
    assert_eq!(pool.total_shares, 0);
    assert_eq!(pool.acc_reward_per_share, 0);
    assert_eq!(pool.last_update_time, 0);
    assert_eq!(pool.paused, false);
    assert_eq!(pool.bump, bump);
}
