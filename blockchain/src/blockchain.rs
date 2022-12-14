use sha256::{digest};
use chrono::*;
use serde::{Serialize, Deserialize};


#[derive(Clone, Serialize, Deserialize)]
pub struct Block{
    pub index:u64,
    pub timestamp:String,
    pub proof:u64,
    pub previous_hash:String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Blockchain{
    pub chain: Vec<Block>,
}
impl  Blockchain{
    pub fn create_block(mut self, proof:u64, previous_hash:String) -> Block{
        let block = Block{
            index: (self.chain.len()+1) as u64,
            timestamp: Utc::now().to_string(),
            proof: proof,
            previous_hash: previous_hash
        };
        self.chain.push(block.clone());
        block
    }
    
    pub fn get_previous_block(self) -> Block{
        let block = self.chain.last().unwrap();
        block.to_owned()
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
    

}
