

use byteorder::{BigEndian, WriteBytesExt,ReadBytesExt};
use std::io::{Cursor};
// use crate::{Reader,PointerType};

#[allow(dead_code)]
pub fn u64_to_be(u:u64)->Vec<u8>{
    u.to_le_bytes().to_vec()
}

#[allow(dead_code)]
pub fn u64_from_bytes(pool:Vec<u8>)->Result<u64,()>{
    // println!("h+ : {:?}",pool);
    let mut rdr = Cursor::new(pool);
    match rdr.read_u64::<BigEndian>(){
        Ok(v)=>{return Ok(v)},
        Err(_e)=>{
            // println!("_e : {:?}",_e);
            return Err(());
        }
    }
}

#[allow(dead_code)]
pub fn u64_to_bytes(n:u64)->Result<Vec<u8>,()>{
    let mut value_len_as_bytes = Vec::new();
    match value_len_as_bytes.write_u64::<BigEndian>(n){
        Ok(_)=>{
            // println!("h- : {:?}",value_len_as_bytes);
            return Ok(value_len_as_bytes);
        },
        Err(_)=>{
            return Err(());
        }
    }
}

#[allow(dead_code)]
pub fn p_error(e:&'static str,p:bool){
    if p {
        println!("!!! {}",e);
    }
}

#[allow(dead_code)]
pub fn print_range(pool:&Vec<u8>,start:usize,len:usize){
    println!("\n\n==================\n\n");
    for i in start..start+len{
        print!(" {:?}",pool[i]);
    }
    println!("\n\n==================\n\n");
}

#[allow(dead_code)]
pub fn buff_print(pool:&Vec<u8>){
    let mut index = 0;
    println!("\n=====================\n");
    for i in pool{
        print!(" | {} : {}",index,i);
        index += 1;
    }
    println!("\n=====================\n");
}

#[allow(dead_code)]
pub fn concat_str(v:String)->String{
    let mut collect = String::new();
    for i in v.chars(){
        if collect.len() == 8{break;}
        collect.push(i)
    }
    return collect;
}