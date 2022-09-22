mod blockchain;
use blockchain::*;

use chrono::*;
use actix_web::{App, get, post, HttpServer, Responder, HttpResponse, services, web::{Data, self}};
use std::{sync::Mutex, collections::HashSet};
use serde_json::{json};


#[get("/mine_block")]
async fn mine_block(blockchain: Data<Mutex<Blockchain>>, node_address: Data<Mutex<NodeAddress>>) -> impl Responder {
    let node_address = node_address.lock().unwrap();
    let mut blockchain = blockchain.lock().unwrap();
    let previous_block = blockchain.chain.last().unwrap().to_owned();
    let previous_proof =previous_block.proof;
    let proof=blockchain.clone().proof_of_work(previous_proof);
    let previous_hash=blockchain.clone().hash(previous_block);

    let transaction = blockchain.clone().add_transaction(node_address.clone().node_address, "miner".to_string(), 1);
    let new_block = blockchain.clone().create_block(proof, previous_hash, transaction);
   
    blockchain.chain.push(new_block.clone());
    HttpResponse::Ok().body(format!("new block mined: {}", serde_json::to_string(&new_block).unwrap()))
}

#[get("/get_chain")]
async fn get_chain(blockchain: Data<Mutex<Blockchain>>) -> impl Responder {
    let blockchain = blockchain.lock().unwrap();
    HttpResponse::Ok().json(
        json!({
            "chain": blockchain.clone().chain,
        })
    )
}

#[get("/is_valid_chain")]
async fn is_valid_chain(blockchain: Data<Mutex<Blockchain>>) -> impl Responder {
    let blockchain = blockchain.lock().unwrap();
    let is_valid=blockchain.clone().is_chain_valid();
    println!("is_valid: {}", is_valid);
    if is_valid{
        HttpResponse::Ok().body(format!( "Chain is Valid" ))
    }
    else{
        HttpResponse::Ok().body(format!( "Chain is not Valid" ))
    }
    
}

#[post("/add_transaction")]
async fn add_transaction(blockchain: Data<Mutex<Blockchain>>, transaction_data:web::Json<Transaction>)-> impl Responder{
    let mut blockchain = blockchain.lock().unwrap();
    
    let transaction = blockchain.clone().add_transaction(transaction_data.sender.clone(), transaction_data.receiver.clone(), transaction_data.amount.clone());
    let index = blockchain.clone().get_previous_block();
    blockchain.chain[index].transaction.push(transaction);

    HttpResponse::Ok().body(format!( "Transaction added to block index: {}",index+1 ))
}

#[post("/connect_node")]
async fn connect_node( node: web::Json<Nodes>, nodes: Data<Mutex<Nodes>>) -> impl Responder {
    
    let mut nodes=nodes.lock().unwrap();
    let node =&node.nodes;
    
    if node.len()>0{
        nodes.nodes = node.to_owned();
        HttpResponse::Ok().body(format!( "failed to add node"))
    }
    else{
        HttpResponse::Ok().body(format!( "Node added"))
    }
    
}

#[post("/replace_chain")]
async fn replace_chain(blockchain: Data<Mutex<Blockchain>>, nodes: Data<Mutex<Nodes>>) -> impl Responder {
    let mut blockchain= blockchain.lock().unwrap();
    let nodes = nodes.lock().unwrap();
    let network=nodes.clone().nodes;
    let chain_len = blockchain.chain.len();
    let client = reqwest::Client::new();

    for node in network{
        let response = client.get(format!("{}/get_chain", node)).send().await.unwrap();
        let chain:Blockchain = response.json().await.unwrap();
        let other_chain_len = chain.chain.len();
        if other_chain_len>chain_len{
            // under development
            blockchain.chain = chain.chain;
        }
    }
    blockchain.clone().replace_chain();

    HttpResponse::Ok().body(format!( "Node added"))
    
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let node_address = uuid::Uuid::new_v4().to_string().replace("-", "");
    let  block = Block{
        index: 1,
        timestamp: Utc::now().to_string(),
        proof: 1,
        previous_hash:"0".to_owned(),
        transaction: vec![]
    };
  
    let data = Data::new(Mutex::new(Blockchain{ chain: vec![block.clone()] }));
    let node_address = Data::new(Mutex::new(NodeAddress{node_address:node_address }));
    let nodes =Data::new(Mutex::new(Nodes{nodes: HashSet::new()}));
    HttpServer::new(move|| {
        App::new()
        .app_data(data.clone())
        .app_data(node_address.clone())
        .app_data(nodes.clone())
        .service(services![ mine_block, get_chain, is_valid_chain, add_transaction, connect_node, replace_chain]) 
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}