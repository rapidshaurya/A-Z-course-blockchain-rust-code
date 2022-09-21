mod blockchain;
use blockchain::*;

use chrono::*;
use actix_web::{App, get, HttpServer, Responder, HttpResponse, services, web::Data};
use std::sync::Mutex;



#[get("/mine_block")]
async fn mine_block(blockchain: Data<Mutex<Blockchain>>) -> impl Responder {
    let mut blockchain = blockchain.lock().unwrap();
    let previous_block = blockchain.chain.last().unwrap().to_owned();
    let previous_proof =previous_block.proof;
    let proof = blockchain::Blockchain::proof_of_work(blockchain::Blockchain { chain: blockchain.chain.clone() }, previous_proof);
    let previous_hash = blockchain::Blockchain::hash(blockchain::Blockchain { chain: blockchain.chain.clone() }, previous_block);
    let new_block = blockchain::Blockchain::create_block(blockchain::Blockchain { chain: blockchain.chain.clone() }, proof, previous_hash);
    blockchain.chain.push(new_block.clone());
    HttpResponse::Ok().body(format!("new block mined: {}", serde_json::to_string(&new_block).unwrap()))
}

#[get("/get_chain")]
async fn get_chain(blockchain: Data<Mutex<Blockchain>>) -> impl Responder {
    let blockchain = blockchain.lock().unwrap();
    let res = blockchain.chain.iter().map(|chain|{ serde_json::to_string(&chain).unwrap() }).collect::<String>();
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


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let  block = Block{
        index: 1,
        timestamp: Utc::now().to_string(),
        proof: 1,
        previous_hash:"0".to_owned(),
    };
    let data = Data::new(Mutex::new(Blockchain{ chain: vec![block.clone()] }));

    HttpServer::new(move|| {
        App::new()
        .app_data(Data::clone(&data))
        .service(services![ mine_block, get_chain, is_valid_chain]) 
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}