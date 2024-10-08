use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
    client::SyncClient,
    rpc_client::RpcClient,
    transaction::TransactionInstruction,
    system_instruction,
};
use std::{fs, error::Error};
// 加载环境变量
fn load_env() {
    dotenv().ok();
}

// 生成或读取付款人的密钥对
fn get_payer() -> Keypair {
    let keypair_file_path = "./payer-keypair.json";

    // 如果密钥对文件存在，则读取
    if fs::metadata(keypair_file_path).is_ok() {
        let secret_key_string = fs::read_to_string(keypair_file_path).expect("Unable to read keypair file");
        let secret_key: Vec<u8> = serde_json::from_str(&secret_key_string).expect("Unable to parse secret key");
        Keypair::from_bytes(&secret_key).expect("Invalid secret key")
    } else {
        // 手动生成付款人的密钥对并保存
        let payer = Keypair::generate();
        let secret_key_json = serde_json::to_string(&payer.to_bytes()).expect("Unable to serialize secret key");
        fs::write(keypair_file_path, secret_key_json).expect("Unable to write keypair file");
        println!("已生成并保存付款人密钥对：{}", payer.pubkey());
        payer
    }
}

// 定义一个异步函数 say_hello
async fn say_hello(client: &RpcClient, payer: &Keypair) -> Result<String, Box<dyn Error>> {
    // 创建交易指令
    let instruction = TransactionInstruction::new_with_bincode(
        Pubkey::from_str("5DBFd6y14vp7ZpAx3CCURMG9w6t23uvgXXxJf2sgWChK").unwrap(),
        &(),
        vec![], // 如果指令需要关联的账户，请在这里添加
    );

    // 创建交易
    let transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    
    // 签署并发送交易
    let (recent_blockhash, _fee_calculator) = client.get_recent_blockhash()?;
    let mut transaction = transaction;
    transaction.sign(&[payer], recent_blockhash);
    
    // 发送并确认交易
    let signature = client.send_and_confirm_transaction(&transaction)?;
    Ok(signature.to_string())
}

// 主程序逻辑
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    load_env(); // 加载环境变量

    // 创建与 Solana 本地集群的连接
    let client = RpcClient::new("http://127.0.0.1:8899");

    // 获取付款人的密钥对
    let payer = get_payer();
    println!("Public Key: {}", payer.pubkey());

    // 请求向付款人地址空投 1 SOL
    let airdrop_signature = client.request_airdrop(&payer.pubkey(), 1_000_000_000)?; // 1 SOL = 1_000_000_000 lamports
    client.confirm_transaction(&airdrop_signature)?;

    println!("Airdrop成功，1 SOL已空投。");

    // 调用 say_hello 函数
    let transaction_signature = say_hello(&client, &payer).await?;
    println!("Transaction: http://127.0.0.1:8899/tx/{}", transaction_signature);
    println!("Finished successfully");

    Ok(())
}