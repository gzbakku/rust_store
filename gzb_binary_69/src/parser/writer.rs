


use crate::workers::{u64_to_bytes};


pub fn init(key:Vec<u8>,value:Vec<u8>)->Result<Vec<u8>,&'static str>{

    let mut collect = vec![0,1,0];
    let mut key = key;
    let mut value = value;

    //key size
    match u64_to_bytes(key.len() as u64){
        Ok(v)=>{
            collect.push(v.len() as u8);
            collect.append(&mut vec![0,2,0]);
            collect.append(&mut v.clone());
            collect.append(&mut vec![0,3,0]);
            collect.append(&mut key);
            collect.append(&mut vec![0,4,0]);
        },
        Err(_)=>{
            return Err("failed-parse-key-len");
        }
    }

    //value size
    match u64_to_bytes(value.len() as u64){
        Ok(mut v)=>{
            collect.push(v.len() as u8);
            collect.append(&mut vec![0,5,0]);
            collect.append(&mut v);
            collect.append(&mut vec![0,6,0]);
            collect.append(&mut value);
            collect.append(&mut vec![0,7,0]);
        },
        Err(_)=>{
            return Err("failed-parse-key-len");
        }
    }

    // println!("sl : {}",collect.len());

    return Ok(collect);

}