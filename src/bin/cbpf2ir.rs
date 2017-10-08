extern crate cbpf;
extern crate cbpf_to_llvm_ir;
#[macro_use]
extern crate error_chain;
extern crate pcap;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use std::fs;
use std::io::{BufWriter, Write};
use cbpf::opcode::BpfInsn;
use cbpf_to_llvm_ir::Converter;
use structopt::StructOpt;

mod errors {
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
            Pcap(::pcap::Error);
        }
    }
}

use errors::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "cbpf2ir", about = "Convert cBPF program to LLVM IR from libpcap's expression")]
struct Opt {
    #[structopt(short = "d", long = "debug", help = "Activate debug mode")] debug: bool,
    #[structopt(short = "o", long = "outfile", help = "Output file")] outfile: String,
    #[structopt(short = "l", long = "linktype", /* default is ethernet */
                help = "LinkType (http://www.tcpdump.org/linktypes.html)", default_value = "1")]
    linktype: i32,
    #[structopt(help = "cBPF filter expression")] expression: String,
}

fn run() -> Result<()> {
    let args = Opt::from_args();

    let pcap = pcap::Capture::dead(pcap::Linktype(args.linktype))?;
    let bpf_prog = pcap.compile(&args.expression)?;
    // we do this since pcap crate does not expose internal bpf structure
    let insns: &[BpfInsn] = unsafe { std::mem::transmute(bpf_prog.get_instructions()) };

    let mut converter = Converter::new();
    let ir = converter.convert(insns);
    if ir.is_err() {
        return Err(format!("{}", ir.err().unwrap()).into());
    }

    if args.debug {
        println!("expression: {}", args.expression);
        println!("length: {:?}", insns.len());
        println!("cBPF program:");
        for insn in insns {
            println!("{:?}", insn);
        }
        println!();
        println!("LLVM IR:");
        converter.dump_module();
    }

    let mut f = BufWriter::new(fs::File::create(args.outfile)?);
    f.write_all(ir.unwrap().as_bytes())?;

    Ok(())
}

quick_main!(run);
