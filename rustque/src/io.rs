
use tokio::fs::{File,OpenOptions};
use std::path::Path;
use tokio::io::{AsyncWriteExt,AsyncReadExt,AsyncSeekExt};
use std::fs::Metadata;
use std::io::{Seek,SeekFrom,Cursor};
// use std::time::Instant;

pub async fn init_map(path:String,frame:u64)->Result<(File,Metadata),&'static str>{

    let build = Path::new(&path);
    if !build.exists() || true{

        //build map with frame size
        let mut file_builder = OpenOptions::new();
        file_builder.write(true)
        .read(true)
        .create(true);
        let mut file:File;
        match file_builder.open(build).await{
            Ok(f)=>{
                file = f;
            },
            Err(_)=>{
                return Err("failed-create_file");
            }
        }

        let mut collect:Vec<u8> = Vec::with_capacity(10000);
        for _ in 0..frame{
            if collect.len() == 10000{
                // println!("filled");
                match file.write_all(&collect).await{
                    Ok(_)=>{
                        // println!("writen");
                    },
                    Err(_)=>{
                        return Err("failed-build_frame-create_file");
                    }
                }
                collect.clear();
            } else {
                collect.push(0);
            }
        }
        if collect.len() > 0{
            match file.write_all(&collect).await{
                Ok(_)=>{},
                Err(_)=>{
                    return Err("failed-build_frame-create_file");
                }
            }
        }

        match file.metadata().await{
            Ok(v)=>{
                return Ok((file,v));
            },
            Err(_)=>{
                return Err("failed-get-metadata");
            }
        }

    }

    match File::open(build).await{
        Ok(file)=>{
            match file.metadata().await{
                Ok(v)=>{
                    return Ok((file,v));
                },
                Err(_)=>{
                    return Err("failed-get-metadata");
                }
            }
        },
        Err(_)=>{
            return Err("failed-open_file");
        }
    }

}

pub async fn expand(file:&mut File,size:&u64)->Result<(),&'static str>{

    match file.seek(SeekFrom::End(0)).await{
        Ok(_)=>{},
        Err(_)=>{
            return Err("failed-seek");
        }
    }

    let mut collect:Vec<u8> = Vec::with_capacity(10000);
    for _ in 0..*size{
        if collect.len() == 10000{
            match file.write_all(&collect).await{
                Ok(_)=>{},
                Err(_)=>{
                    return Err("failed-build_frame-create_file");
                }
            }
            collect.clear();
        } else {
            collect.push(0);
        }
    }
    if collect.len() > 0{
        match file.write_all(&collect).await{
            Ok(_)=>{},
            Err(_)=>{
                return Err("failed-build_frame-create_file");
            }
        }
    }

    return Ok(());

}

pub async fn read_chunk(file:&mut File,buffer:&mut Vec<u8>,start_at:u64,read_len:u64)->Result<usize,&'static str>{

    match file.seek(SeekFrom::Start(start_at)).await{
        Ok(_)=>{},
        Err(_e)=>{
            return Err("failed-seek");
        }
    }

    match file.take(read_len).read_to_end(buffer).await{
        Ok(v)=>{
            return Ok(v);
        },
        Err(e)=>{
            // println!("!!! failed-read-io => {:?}",e);
            return Err("failed-read_chunk");
        }
    }

}