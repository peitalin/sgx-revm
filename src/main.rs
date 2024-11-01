use alloy_primitives::{Address, U256};
use revm::{
    primitives::{AccountInfo, TxEnv},
    InMemoryDB,
    Evm,
    db::EmptyDB,
    Context,
    EvmContext,
    inspectors::NoOpInspector,
};

use std::{io::Read, net::TcpListener};

mod server;
use server::get_key_and_cert;

// This payload should be generalized to include all the pre-state for each
// simulation.
#[derive(serde::Deserialize)]
struct Payload {
    sender: Address,
    amount: U256,
}

fn main() -> eyre::Result<()> {
    let (key, cert) = get_key_and_cert();

    println!("test REVM tx: simulate(Payload)");

    simulate(Payload {
        sender: "0xdafea492d9c6733ae3d56b7ed1adb60692c98bc5".parse()?,
        amount: U256::from(155),
    })?;

    // dbg!(&cert);
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Listening on 127.0.0.1:7878");


    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buf = vec![];
        let _num_bytes = stream.read_to_end(&mut buf)?;
        let data: Payload = serde_json::from_slice(&buf)?;
        println!("recv: sender: {}", data.sender);
        println!("recv: amount: {}", data.amount);
        simulate(data)?;

        // TODO: Re-enable this,
        // let _ = serve(stream, &mut key, &mut cert).unwrap();
    }

    Ok(())
}

fn simulate(payload: Payload) -> eyre::Result<()> {
    let mut db = InMemoryDB::default();
    let receiver = payload.sender;
    let value = payload.amount;

    let balance = U256::from(500);
    // this is a random address
    let address = "0x4838b106fce9647bdf1e7877bf73ce8b0bad5f97".parse()?;
    let info = AccountInfo {
        balance,
        ..Default::default()
    };

    // Populate the DB pre-state,
    // TODO: Make this data witnessed via merkle patricia proofs.
    db.insert_account_info(address, info);
    // For storage insertions:
    // db.insert_account_storage(address, slot, value)

    let tx_env= TxEnv {
        caller: address,
        transact_to: revm::primitives::TransactTo::Call(revm::primitives::Address::from(receiver.0.0)),
        value,
        ..Default::default()
    };

    // Setup the EVM with the configured DB
    // The EVM will ONLY be able to access the witnessed state, and
    // any simulation that tries to use state outside of the provided data
    // will fail.
    // let mut evm = Evm::new();
    let mut evm = Evm::builder()
        .with_db(db)
        .with_tx_env(tx_env)
        .build();

    let result = evm.transact()?;

    assert_eq!(
        result.state.get(&address).unwrap().info.balance,
        U256::from(345)
    );

    dbg!(&result);

    Ok(())
}
