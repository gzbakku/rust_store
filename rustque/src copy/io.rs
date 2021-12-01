
use tokio::fs::{File,OpenOptions};
use std::path::Path;
use tokio::io::{AsyncWriteExt,AsyncReadExt,AsyncSeekExt};
use std::fs::Metadata;
use std::io::{SeekFrom};
use crate::workers::debug_error;

const ERROR:bool = true;

pub async fn init_map(path:String,frame:u64)->Result<(File,Metadata),&'static str>{

    let build = Path::new(&path);
    if !build.exists() || false{

        //build map with frame size
        let mut file_builder = OpenOptions::new();
        file_builder.write(true)
        .read(true)
        .create(true);
        // .create_new(true);
        let mut file:File;
        match file_builder.open(build).await{
            Ok(f)=>{
                file = f;
            },
            Err(_)=>{
                debug_error("failed-create_file-init_map-io.rs",ERROR);
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
                        debug_error("failed-write_block-expand_file-init_map-io.rs",ERROR);
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
                    debug_error("failed-expand_file-last-init_map-io.rs",ERROR);
                    return Err("failed-build_frame-create_file");
                }
            }
        }

        match file.metadata().await{
            Ok(v)=>{
                return Ok((file,v));
            },
            Err(_)=>{
                debug_error("failed-get_metadata-init_map-io.rs",ERROR);
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
                    debug_error("failed-existing-get_metadata-init_map-io.rs",ERROR);
                    return Err("failed-get-metadata");
                }
            }
        },
        Err(_)=>{
            debug_error("failed-open_file-init_map-io.rs",ERROR);
            return Err("failed-open_file");
        }
    }

}

pub async fn expand(file:&mut File,size:&u64)->Result<(),&'static str>{

    match file.seek(SeekFrom::End(0)).await{
        Ok(_)=>{},
        Err(_)=>{
            debug_error("failed-file_seek-expand-io.rs",ERROR);
            return Err("failed-seek");
        }
    }

    let mut collect:Vec<u8> = Vec::with_capacity(10000);
    for _ in 0..*size{
        if collect.len() == 10000{
            match file.write_all(&collect).await{
                Ok(_)=>{},
                Err(_)=>{
                    debug_error("failed-write_block-expand-io.rs",ERROR);
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
                debug_error("failed-write_block-last-expand-io.rs",ERROR);
            }
        }
    }

    return Ok(());

}

pub async fn write_chunk(file:&mut File,start_at:u64,buffer:Vec<u8>)->Result<(),&'static str>{

    match file.seek(SeekFrom::Start(start_at)).await{
        Ok(_)=>{
            // println!("k : {}",k);
        },
        Err(_e)=>{
            debug_error("failed-file_seek-write_chunk-io.rs",ERROR);
            return Err("failed-seek");
        }
    }

    match file.write_all(&buffer).await{
        Ok(_)=>{
            return Ok(());
        },
        Err(_)=>{
            debug_error("failed-write_all-write_chunk-io.rs",ERROR);
            // println!("!!! failed-read-io => {:?}",e);
            return Err("failed-read_chunk");
        }
    }

}

pub async fn read_chunk(file:&mut File,buffer:&mut Vec<u8>,start_at:u64,read_len:u64)->Result<usize,&'static str>{

    match file.seek(SeekFrom::Start(start_at)).await{
        Ok(_)=>{},
        Err(_e)=>{
            debug_error("failed-seek-read_chunk-io.rs",ERROR);
            return Err("failed-seek");
        }
    }

    match file.take(read_len).read_to_end(buffer).await{
        Ok(v)=>{
            return Ok(v);
        },
        Err(_)=>{
            debug_error("failed-take-read_chunk-io.rs",ERROR);
            // println!("!!! failed-read-io => {:?}",e);
            return Err("failed-read_chunk");
        }
    }

}

pub async fn read_full(path:String){

    // println!("one");

    let mut file_builder = OpenOptions::new();
    file_builder
    .write(true)
    .read(true)
    .create(true);
    let mut file:File;
    match file_builder.open(path).await{
        Ok(f)=>{
            file = f;
        },
        Err(_)=>{
            println!("!!! failed-open_file");
            return;
        }
    }

    // println!("two");

    let mut buffer = Vec::new();
    match file.seek(SeekFrom::Start(0)).await{
        Ok(_)=>{},
        Err(_)=>{
            println!("!!! failed-seek_file");
            return;
        }
    }

    // println!("three");

    match file.read_to_end(&mut buffer).await{
        Ok(_)=>{
            println!("\n{:?}\n",buffer);
        },
        Err(_)=>{
            println!("!!! failed-read_file");
            return;
        }
    }

}