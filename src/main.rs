use std::{net::IpAddr, path::PathBuf};
use alsa::device_name;
use vban_sink::vban;
use clap::Parser;

/**
 * Notes:
 * ALSA buffer may be tweaked via hardware and software parameters, namely pcm.sw_params_current() or pcm.hw_params_current(). The swp.set_start_threshold(x) may be used to determine the amount of frames that have to be available in order for playback to start. 
 * 
 * ToDo: 
 * - Support multiple sample rates
 * - Support multiple sample formats
 * - Check and discriminate stream names
 * - Support config files (if necessary)
 */



#[derive(Parser)]
struct Cli {
    /// Specify an IP-address if you don't want to bind to all interfaces
    addr : Option<IpAddr>,

    /// Specify a different port if you don't want to use port 6980
    #[arg(short, long)]
    port : Option<u16>,

    /// Use a config file
    #[arg(short, long, value_name = "file")]
    config: Option<PathBuf>,

    /// Specify a stream name if you want the application to discriminate incoming streams
    #[arg(short, long, value_name = "name")]
    stream_name : Option<String>,

    /// Prepend silence when starting playback. Supply duration in milliseconds.
    #[arg(short='x', long, value_name = "duration")]
    silence : Option<u32>,

    /// Name of the audio device that is used as a sink (default is "default")
    #[arg(short, long)]
    device_name : Option<String>,
}

// #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn main() -> Result<(), i32> {

    let cli = Cli::parse();

    let use_config = match cli.config {
        None => false,
        Some(_) => panic!("Config files are currently not supported."),
    };

    let addr : IpAddr;
    let port : u16;
    let stream_name : Option<String>;
    let mut device_name = String::from("default");
    
    if use_config {
        // todo 
        addr = "127.0.0.1".parse().unwrap();
        port = 6980;
        stream_name = None;
    } else {
        addr = match cli.addr {
            None => "0.0.0.0".parse().unwrap(),
            Some(addr) => {
                println!("Using {addr} as address to bind to.");
                addr
            },
        };
        port = match cli.port {
            None => 6980,
            Some(num) => {
                println!("Using port {num}.");
                num
            },
        };
        stream_name = match cli.stream_name {
            None => None,
            Some(name) => {
                println!("Using {name} as stream name.");
                Some(name)
            },
        };
        device_name = match cli.device_name {
            None => String::from("default"),
            Some(name) => name,
        };
    }


    let mut vbr = match vban::VbanRecipient::create(
    addr, port, stream_name, None, None,
    device_name, cli.silence){
        None => {
            println!("Could not create VBAN recipient.");
            return Err(-1)
        },
        Some(_vbr) => {
            _vbr
        }
    };


    loop {
        vbr.handle();
    }

}
