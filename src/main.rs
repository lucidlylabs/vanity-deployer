use alloy::{
    primitives::{Address, FixedBytes},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};
use rayon::prelude::*;
use std::str::FromStr;
use std::{env, fs};

alloy::sol!(
    #[sol(rpc)]
    interface Deployer {
        function deployContract(string memory name, bytes memory creationCode, bytes memory constructor, uint256 value) external returns (address);
    }
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let rpc_url = env::var("RPC_URL").expect("RPC_URL not included in .env");
    let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY is not included in the .env");
    let _deployer_address = Address::from_str(
        &env::var("DEPLOYER_ADDRESS").expect("DEPLOYER_ADDRESS is not included in the .env"),
    )?;
    let _desired_prefix = "TxBundler".to_lowercase();

    let _creation_code_hex = fs::read_to_string("./contracts/DeployerContractCreationCode.txt")
        .expect("DeployerContractCreationCode.txt file not found.")
        .trim()
        .to_string();
    let _creation_code = hex::decode(&_creation_code_hex)?;
    let _creation_code_hash = FixedBytes::<32>::from(alloy::primitives::keccak256(&_creation_code));

    let _provider = ProviderBuilder::new().connect(&rpc_url).await?;
    let _signer: PrivateKeySigner = private_key.parse().expect("expecting a private key");

    let (name, address) =
        find_vanity_address(&_deployer_address, &_desired_prefix, &_creation_code_hash)?;

    println!("found name: {}", name);
    println!("address: {}", address.to_checksum(None));

    Ok(())
}

fn compute_create3_address(
    deployer_address: &Address,
    name: &str,
    creation_code_hash: &FixedBytes<32>,
) -> Address {
    let encoded_name = alloy::sol_types::SolValue::abi_encode(&name);
    let salt = FixedBytes::<32>::from(alloy::primitives::keccak256(&encoded_name));

    // compute CREATE3 address: keccak256(0xff ++ deployerAddress ++ salt ++ keccak256(creationCode))[12:]
    let create3_input = [
        &[0xff_u8][..],
        deployer_address.as_slice(),
        salt.as_slice(),
        creation_code_hash.as_slice(),
    ]
    .concat();
    let hash = alloy::primitives::keccak256(&create3_input);
    Address::from_slice(&hash[12..])
}

fn find_vanity_address(
    deployer_address: &Address,
    desired_prefix: &str,
    creation_code_hash: &FixedBytes<32>,
) -> Result<(String, Address), Box<dyn std::error::Error>> {
    let base_name = "Transaction Bundler V1.0";
    const CHUNK_SIZE: u64 = 1000;
    const MAX_COUNTER: u64 = 1_000_000_000; // Large but finite range

    let result = (0..MAX_COUNTER / CHUNK_SIZE)
        .into_par_iter()
        .find_any(|&chunk| {
            let start = chunk * CHUNK_SIZE;
            for counter in start..start + CHUNK_SIZE {
                let name = format!("{}{}", base_name, counter);
                let address = compute_create3_address(deployer_address, &name, creation_code_hash);
                if counter % 10000 == 0 {
                    println!("Checked {} names...", counter);
                }
                if address
                    .to_checksum(None)
                    .to_lowercase()
                    .starts_with(desired_prefix)
                {
                    return Some(counter);
                }
            }
            None::<u64>
        });

    match result {
        Some(counter) => {
            let name = format!("{}{}", base_name, counter);
            let address = compute_create3_address(deployer_address, &name, creation_code_hash);
            Ok((name, address))
        }
        None => Err("No matching address found within range".into()),
    }
}
