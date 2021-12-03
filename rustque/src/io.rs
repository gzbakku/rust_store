
use tokio::fs::{File,OpenOptions};
use std::path::Path;
use tokio::io::{AsyncWriteExt,AsyncReadExt,AsyncSeekExt};
use std::fs::Metadata;
use std::io::{SeekFrom};
// use crate::workers::{debug_error,debug_message};
// use std::time::Instant;

// const DEBUG:bool = false;
// const ERROR:bool = true;

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
                // debug_error("failed-create_file-init_map-io.rs",ERROR);
                return Err("failed-create_file");
            }
        }

        let mut collect:Vec<u8> = Vec::with_capacity(10000);
        for _ in 0..frame{
            if collect.len() == 10000{
                // debug_message("filled",DEBUG);
                match file.write_all(&collect).await{
                    Ok(_)=>{
                        // debug_message("writen",DEBUG);
                    },
                    Err(_)=>{
                        // debug_error("failed-write_block-expand_file-init_map-io.rs",ERROR);
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
                    // debug_error("failed-expand_file-last-init_map-io.rs",ERROR);
                    return Err("failed-build_frame-create_file");
                }
            }
        }

        match file.metadata().await{
            Ok(v)=>{
                return Ok((file,v));
            },
            Err(_)=>{
                // debug_error("failed-get_metadata-init_map-io.rs",ERROR);
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
                    // debug_error("failed-existing-get_metadata-init_map-io.rs",ERROR);
                    return Err("failed-get-metadata");
                }
            }
        },
        Err(_)=>{
            // debug_error("failed-open_file-init_map-io.rs",ERROR);
            return Err("failed-open_file");
        }
    }

}

pub async fn expand(file:&mut File,size:&u64)->Result<(),&'static str>{

    // let hold = Instant::now();
    match file.seek(SeekFrom::End(0)).await{
        Ok(_)=>{},
        Err(_)=>{
            // debug_error("failed-file_seek-expand-io.rs",ERROR);
            return Err("failed-seek");
        }
    }

    let mut collect:Vec<u8> = Vec::with_capacity(1000000);
    for _ in 0..*size{
        if collect.len() == 1000000{
            match file.write(&collect).await{
                Ok(_)=>{},
                Err(_)=>{
                    // debug_error("failed-write_block-expand-io.rs",ERROR);
                    return Err("failed-build_frame-create_file");
                }
            }
            collect.clear();
        } else {
            collect.push(0);
        }
    }
    if collect.len() > 0{
        match file.write(&collect).await{
            Ok(_)=>{},
            Err(_)=>{
                // debug_error("failed-write_block-last-expand-io.rs",ERROR);
            }
        }
    }

    // println!("expand in : {:?}",hold.elapsed());

    return Ok(());

}

pub async fn remove_chunk(file:&mut File,start_at:usize,len:usize)->Result<(),&'static str>{

    match file.seek(SeekFrom::Start(start_at as u64)).await{
        Ok(_)=>{},
        Err(_e)=>{
            return Err("failed-seek");
        }
    }

    let mut collect:Vec<u8> = Vec::with_capacity(len as usize);
    for _ in 0..len{
        collect.push(0);
    }

    match file.write(&collect).await{
        Ok(_)=>{
            return Ok(());
        },
        Err(_)=>{
            return Err("failed-build_frame-create_file");
        }
    }

}

pub async fn write_chunk(file:&mut File,start_at:u64,buffer:Vec<u8>)->Result<(),&'static str>{

    // let hold = Instant::now();
    match file.seek(SeekFrom::Start(start_at)).await{
        Ok(_)=>{},
        Err(_e)=>{
            // debug_error("failed-file_seek-write_chunk-io.rs",ERROR);
            return Err("failed-seek");
        }
    }
    
    match file.write(&buffer).await{
        Ok(_)=>{
            // println!("write in : {:?}",hold.elapsed());
            // debug_message("chunk writen",DEBUG);
            return Ok(());
        },
        Err(_)=>{
            // debug_error("failed-write_all-write_chunk-io.rs",ERROR);
            return Err("failed-read_chunk");
        }
    }

}

pub async fn read_chunk(file:&mut File,buffer:&mut Vec<u8>,start_at:u64,read_len:u64)->Result<usize,&'static str>{

    match file.seek(SeekFrom::Start(start_at)).await{
        Ok(_)=>{},
        Err(_e)=>{
            // debug_error("failed-seek-read_chunk-io.rs",ERROR);
            return Err("failed-seek");
        }
    }

    match file.take(read_len).read_to_end(buffer).await{
        Ok(v)=>{
            return Ok(v);
        },
        Err(_)=>{
            // debug_error("failed-take-read_chunk-io.rs",ERROR);
            return Err("failed-read_chunk");
        }
    }

}

#[allow(dead_code)]
pub async fn read_full(path:String){

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
            // debug_error("failed-open_file",ERROR);
            return;
        }
    }

    let mut buffer = Vec::new();
    match file.seek(SeekFrom::Start(0)).await{
        Ok(_)=>{},
        Err(_)=>{
            // debug_error("failed-seek_file",ERROR);
            return;
        }
    }

    match file.read_to_end(&mut buffer).await{
        Ok(_)=>{},
        Err(_)=>{
            // debug_error("failed-read_file",ERROR);
            return;
        }
    }

}