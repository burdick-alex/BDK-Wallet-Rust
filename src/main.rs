#![allow(warnings)]
use bdk::{FeeRate, Wallet,SyncOptions};
use bdk::database::MemoryDatabase;
use bdk::wallet::AddressIndex::New;
use bdk::bitcoin::Network::Testnet;
use bdk::descriptor::Descriptor;

use bdk::keys;
use bdk::bitcoin::util::bip32::ExtendedPrivKey;

use bitcoin::consensus::serialize;
use bdk::bitcoin::Address;
use std::str::FromStr;
use bdk::SignOptions;
use bdk::bitcoin::util::address::Error;
use bdk::blockchain::ElectrumBlockchain;
use bdk::electrum_client::Client;
use bdk::blockchain::Blockchain;
use bdk::descriptor;
use bdk::keys::ExtendedKey;


use bip39::{Mnemonic,Language};
use rand::{thread_rng};

use bdk::template::Bip84;
use bdk::KeychainKind;

use bdk::wallet::AddressIndex::LastUnused;

use std::io;
use std::io::Write;

use std::env;

fn makeMnemonic() -> Mnemonic {
    let seedd = env!("SEED","Please set SEED in .env file.");
    //println!("seed: {}", seedd.to_string());
    let mut rng = thread_rng();
    let m = Mnemonic::generate_in_with(&mut rng, Language::English, 12).unwrap();//.to_seed("hello friend");
    let m2 = Mnemonic::from_str(seedd).unwrap();
    // let key = "SEED";
    // env::set_var(key, m2.to_string());
    m2
}

fn makeXprv() -> ExtendedPrivKey {
    let m = makeMnemonic().to_string();
    let seed = m.as_bytes();
    let xprv = ExtendedPrivKey::new_master(bdk::bitcoin::Network::Testnet, seed).unwrap();
    xprv
}

fn transaction(amount:u64,wallet:&Wallet<MemoryDatabase>,address:&str) -> Result<bool,bdk::Error> {
    println!("Addr: {}!", address);
    println!("Amount: {}!", amount);

    let client = Client::new("ssl://electrum.blockstream.info:60002")?;
    let blockchain = ElectrumBlockchain::from(client); 
    let balance = wallet.get_balance()?;
    //set the address from a string
    let faucet_address = Address::from_str(address).unwrap();
    //build the transaction
    let mut tx_builder = wallet.build_tx();
    tx_builder
        .add_recipient(faucet_address.script_pubkey(), amount)
        .enable_rbf()
        .fee_rate(FeeRate::from_sat_per_vb(5.0))
        .do_not_spend_change();
    
    let (mut psbt, tx_details) = tx_builder.finish().unwrap();

    println!("Transaction details: {:#?}", tx_details);
    println!("Transaction details: {:#?}", psbt);

    //sign transaction
    let psbt_is_finalized = wallet.finalize_psbt(&mut psbt, SignOptions::default())?;
    let finalized = wallet.sign(&mut psbt, SignOptions::default())?;

    //check for finalization
    println!("{}",finalized);
    assert!(finalized, "Tx has not been finalized");

    //broadcast to blockchain
    let raw_transaction = psbt.extract_tx();
    let txid = blockchain.broadcast(&raw_transaction)?;


    println!("Transaction sent! TXID: {:#?}",txid);
    Ok(true)
}


fn printBalance(wallet:&Wallet<MemoryDatabase>) -> Result<String,bdk::Error> {
    let client = Client::new("ssl://electrum.blockstream.info:60002")?;
    let blockchain = ElectrumBlockchain::from(client);

    wallet.sync(&blockchain, SyncOptions::default())?;
    let balance = wallet.get_balance()?;
    println!("Descriptor balance: {} SAT", balance);
    Ok(balance.to_string())
}

fn main() -> Result<(), bdk::Error> {

    let xprv = makeXprv();

    let wallet = Wallet::new(
        Bip84(xprv.clone(), KeychainKind::External),
        Some(Bip84(xprv, KeychainKind::Internal)),
        bdk::bitcoin::Network::Testnet,
        MemoryDatabase::default()
    )?;

    //println!("secret key: {}", xprv.to_string());
    //println!("secret key: {}", wallet.public_descriptor(KeychainKind::External)?.unwrap().to_string());
    //let s = Bip84(xprv.clone(), KeychainKind::External);
    //println!("secret key: {}", wallet.get_descriptor_for_keychain(KeychainKind::External));
    let client = Client::new("ssl://electrum.blockstream.info:60002")?;
    let blockchain = ElectrumBlockchain::from(client); 
    wallet.sync(&blockchain, SyncOptions::default())?;
    printBalance(&wallet);

    //transaction(20000,&wallet,"2N993ukWfRiCE7T7xzhf9nAULupqw4h5qvj");

    while true {
        let mut line = String::new();
        print!("Options:\nBalance\nSend\nRecieve\nWhat would you like to do:");
        io::stdout().flush().unwrap();
        let b1 = std::io::stdin().read_line(&mut line).unwrap();
        line =  line.to_lowercase().trim().to_string();
        if(line == "send")
        {
            let mut addr = String::new();
            let mut amt = String::new();
            println!("\n\n\nSend:");
            print!("\n\n\n--------------------------\nHow much would you like to send(number in sats):");
            io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut amt).unwrap();
            let amount: u64 = amt.trim().parse().expect("Wanted a number");
            //let my_int: i64 = amount.parse().unwrap();

            print!("\nEnter destination address:");
            io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut addr).unwrap();
            let address =  addr.trim().to_string();



            println!("Addr: {}!", address);
            println!("Amount: {}!", amount);

            let didItWork = transaction(amount,&wallet,&address).unwrap();

            // if(didItWork)
            // {
            //     println!("Transaction Successful :)");
            // }
            // else
            // {
            //     println!("Transaction Not Successful :(");
            // }

            println!("Transaction: {}",didItWork);

            




        }
        else if(line == "recieve")
        {
            println!("\n\n\nRecieve:");
            println!("Here is an Address: {}", wallet.get_address(LastUnused)?);
        }
        else if(line == "balance")
        {
            println!("\n\n\nBalance:");
            wallet.sync(&blockchain, SyncOptions::default())?;
            printBalance(&wallet);

        }
        else
        {
            println!("Balance2 , {}", line);
        }




    }


    
    //println!("Address #0: {}", wallet.get_address(LastUnused)?);///////////////////////////////////use this for new address


    Ok(())
}



















// // println!("Address #1: {}", wallet.get_address(New)?);
// // println!("Address #2: {}", wallet.get_address(New)?);
//println!("Descriptor balance: {} SAT", wallet.get_balance()?);
//let didItWork = transaction(1000,&wallet,"2Mt16k3q1CCHPvkFUTnjCBgqYL9sPvF1V7j");
// println!("secret key: {}", wallet.get_address(LastUnused)?.to_string());
// let balance = wallet.get_balance()?;
// let faucet_address = Address::from_str("2Mt16k3q1CCHPvkFUTnjCBgqYL9sPvF1V7j").unwrap();
// let mut tx_builder = wallet.build_tx();
// tx_builder
//     .add_recipient(faucet_address.script_pubkey(), balance /2)
//     .enable_rbf()
//     .fee_rate(FeeRate::from_sat_per_vb(5.0))
//     .do_not_spend_change();
// let (mut psbt, tx_details) = tx_builder.finish().unwrap();
// println!("Transaction details: {:#?}", tx_details);
// println!("Transaction details: {:#?}", psbt);
// let psbt_is_finalized = wallet.finalize_psbt(&mut psbt, SignOptions::default())?;
// let finalized = wallet.sign(&mut psbt, SignOptions::default())?;
// println!("{}",finalized);
// assert!(finalized, "Tx has not been finalized");
// let raw_transaction = psbt.extract_tx();
// let txid = blockchain.broadcast(&raw_transaction)?;
// // println!(
// //     "Transaction sent! TXID: {txid}.\nExplorer URL: https://blockstream.info/testnet/tx/{txid}",
// //     txid = txid
// // );
// wallet.sync(&blockchain, SyncOptions::default())?;
// //let ad = wallet.get_address(New)?;
// println!("Descriptor balance: {} SAT", wallet.get_balance()?);


