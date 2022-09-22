


use sha256::{digest};
use chrono::*;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;

#[derive(Clone, Serialize, Deserialize)]
pub struct NodeAddress{
    pub node_address:String
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Nodes{
    pub nodes:HashSet<String>
}


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Transaction{
    pub sender:String,
    pub receiver: String,
    pub amount: u64 
}


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block{
    pub index:u64,
    pub timestamp:String,
    pub proof:u64,
    pub previous_hash:String,
    pub transaction: Vec<Transaction>
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Blockchain{
    pub chain: Vec<Block>,
}
impl Blockchain{
    pub fn create_block(mut self, proof:u64, previous_hash:String, transaction:Transaction) -> Block{
        let block = Block{
            index: (self.chain.len()+1) as u64,
            timestamp: Utc::now().to_string(),
            proof: proof,
            previous_hash: previous_hash,
            transaction: vec![transaction]
        };
        self.chain.push(block.clone());
        block
    }
    
    pub fn get_previous_block(self) -> usize{
        self.chain.len() -1
    }

    pub fn proof_of_work(self, previous_proof:u64) -> u64{
        let mut new_proof:u64 =1;
        let mut check_proof=false;
        while check_proof == false{
            if new_proof>=previous_proof {
                let input = new_proof.pow(2) - previous_proof.pow(2);
                let input_to_str = input.to_string();
                let hash_operation = digest(input_to_str);
                let a:String=hash_operation.clone().chars().take(4).collect();
                println!("{}",&a);
                if a.trim() =="0000"{
                    check_proof=true;
                }
                else {
                    new_proof= new_proof+1;
                }
            }else {
                new_proof= new_proof+1;
            }
           
        }
        return new_proof;

    }

    pub fn hash(self, block:Block) ->String {
        let data_to_create_hash=serde_json::to_string(&block).unwrap();
        digest(data_to_create_hash)
    }

    pub fn is_chain_valid(self) -> bool{

        let mut previous_block= self.chain[0].clone();
        let mut block_index=1;
        let mut res=false;
        while block_index < self.clone().chain.len() {
            let block = self.clone().chain[block_index].clone();
            if block.previous_hash != self.clone().hash(previous_block.clone()) {
                res = false;
                break;
            }
            let previous_proof = previous_block.proof;
            let proof = block.proof;
            let input_to_str = (proof.pow(2) - previous_proof.pow(2)).to_string();
            let hash_operation = digest(input_to_str);
            let a:String=hash_operation.clone().chars().take(4).collect();
                if a.trim() !="0000"{
                    res = false;
                    break;
                }
                previous_block = block;
                block_index += 1;
                res = true;

        
        }
        
        res
    }

    pub fn add_transaction(self, sender:String, receiver: String, amount: u64)-> Transaction{
        let transaction =Transaction{
            sender:sender,
            receiver:receiver,
            amount:amount
        };
        transaction
    }
    pub fn replace_chain(self){
    }

   
    

}







use actix_web::{App, get, post, HttpServer, Responder, HttpResponse, services, web::{Data, self}};
use std::{sync::Mutex};
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
        HttpResponse::Ok().body(format!( "Node added"))
    }
    else{
        println!("{}", node.len());
        HttpResponse::Ok().body(format!( "Node not added"))
    }
    
}

#[post("/replace_chain")]
async fn replace_chain(blockchain: Data<Mutex<Blockchain>>, nodes: Data<Mutex<Nodes>>) -> impl Responder {
    let mut blockchain= blockchain.lock().unwrap();
    let nodes = nodes.lock().unwrap();
    let network=nodes.clone().nodes;
    let chain_len = blockchain.chain.len();
    let client = reqwest::Client::new();
    let mut flag = false;
    println!("before loop");
    for node in network{
        println!("inside loop {}", node);
        println!("{}",format!("{}/get_chain", node));
        let response = client.get(format!("{}/get_chain", node)).send().await.unwrap();
        println!("response sended");
        let chain:Blockchain = response.json().await.unwrap();
        let other_chain_len = chain.chain.len();
        if other_chain_len>chain_len{
            // under development
            blockchain.chain = chain.chain;
            flag=true;
        }
    }
    if flag{

        HttpResponse::Ok().body(format!( "chain replaced"))

    }
    else{
        HttpResponse::Ok().body(format!( "your chain is longest chain in network"))
    }
    
    
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
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}