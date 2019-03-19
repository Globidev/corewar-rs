extern crate corewar;

use corewar::vm::VirtualMachine;
// use corewar::vm::types::PlayerId;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    champion_files: Vec<String>
}

fn main() -> Result<(), io::Error> {
    let opts = Options::from_args();

    println!("{:?}", opts);

    if opts.champion_files.len() < 1 {
        panic!("Require at least 1 champion");
    }

    let players = opts.champion_files.iter()
        .enumerate()
        .map(|(i, file_name)| {
            let champion = read_cor_file(&file_name)?;
            Ok((i as i32 + 1, champion))
        })
        .collect::<Result<Vec<_>, io::Error>>()?;

    let mut vm = VirtualMachine::new();
    vm.load_players(&players);

    while !vm.processes.is_empty() {
        vm.tick();
        if vm.cycles % 500 == 0 {
            println!("{}", vm.cycles);
            println!("{}", vm.processes.len());
        }
    }

    Ok(())
}

use std::fs::File;
use std::io::{self, Read};

fn read_cor_file(file_name: &str) -> Result<Vec<u8>, io::Error> {
    let mut data = Vec::with_capacity(8192);
    let mut file = File::open(file_name)?;
    file.read_to_end(&mut data)?;
    Ok(data)
}
