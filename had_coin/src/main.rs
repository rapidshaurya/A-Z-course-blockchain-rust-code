mod blockchain;
use blockchain::*;

use chrono::*;
use actix_web::{App, get, post, HttpServer, Responder, HttpResponse, services, web::{Data, self}};
use std::sync::Mutex;



#[get("/mine_block")]
async fn mine_block(blockchain: Data<Mutex<Blockchain>>, node_address: Data<Mutex<NodeAddress>>) -> impl Responder {
    let node_address = node_address.lock().unwrap();
    let mut blockchain = blockchain.lock().unwrap();
    let previous_block = blockchain.chain.last().unwrap().to_owned();
    let previous_proof =previous_block.proof;
    let proof=blockchain.clone().proof_of_work(previous_proof);
    let previous_hash=blockchain.clone().hash(previous_block);

    let transaction = blockchain.clone().add_transaction(node_address.clone().node_address, "miner".to_string(), 1);
  
    let new_block = blockchain::Blockchain::create_block(blockchain::Blockchain { chain: blockchain.chain.clone() }, proof, previous_hash, transaction);
    blockchain.chain.push(new_block.clone());
    HttpResponse::Ok().body(format!("new block mined: {}", serde_json::to_string(&new_block).unwrap()))
}

#[get("/get_chain")]
async fn get_chain(blockchain: Data<Mutex<Blockchain>>) -> impl Responder {
    let blockchain = blockchain.lock().unwrap();
    let res = blockchain.chain.iter().map(|chain|{ serde_json::to_string_pretty(&chain).unwrap() }).collect::<String>();
    HttpResponse::Ok().body(res)
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
    HttpServer::new(move|| {
        App::new()
        .app_data(Data::clone(&data))
        .app_data(node_address.clone())
        .service(services![ mine_block, get_chain, is_valid_chain, add_transaction]) 
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}