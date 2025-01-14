/**
 * Notes:
 * ALSA buffer may be tweaked via hardware and software parameters, namely pcm.sw_params_current() or pcm.hw_params_current(). The swp.set_start_threshold(x) may be used to determine the amount of frames that have to be available in order for playback to start. 
 */


use vban_sink::vban::{self, AlsaSink};

// #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn main() -> Result<(), i32> {

    let mut vbr = match vban::VbanRecipient::create(
        "127.0.0.1".parse().unwrap(), 
        6980, 
        String::from("esp32"), 
        Some(2),
        None){
            None => {
                println!("Could not create VBAN recipient.");
                return Err(-1)
            },
            Some(_vbr) => {
                _vbr
            }
    };

    let sink = AlsaSink::init("pipewire", Some(2), Some(44100)).unwrap();


    loop {
        vbr.handle(&sink);
    }

    Ok(())
}
