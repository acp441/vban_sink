
pub mod vban{
    use core::panic;
    use std::{net::{IpAddr, UdpSocket}, str::from_utf8, time::{ Duration, Instant}, usize};
    use alsa::{pcm::*, ValueOr};
    use alsa::Direction;
    use byteorder::{ByteOrder, LittleEndian};


    const VBAN_HEADER_SIZE : usize = 4 + 1 + 1 + 1 + 1 + 16;
    const VBAN_STREAM_NAME_SIZE : usize = 16;
    const VBAN_PROTOCOL_MAX_SIZE : usize = 1464;
    const VBAN_DATA_MAX_SIZE : usize = VBAN_PROTOCOL_MAX_SIZE - VBAN_HEADER_SIZE;
    const VBAN_CHANNELS_MAX_NB : usize = 256;
    const VBAN_SAMPLES_MAX_NB : usize = 256;


    const VBAN_PACKET_NUM_SAMPLES : usize = 256;  
    const VBAN_PACKET_MAX_SAMPLES : usize = 1024;
    // const VBAN_PACKET_MAX_SAMPLES : usize = 256;
    const VBAN_PACKET_HEADER_BYTES : usize = 24;  
    const VBAN_PACKET_COUNTER_BYTES : usize = 4;  
    const VBAN_PACKET_MAX_LEN_BYTES : usize = VBAN_PACKET_HEADER_BYTES + VBAN_PACKET_COUNTER_BYTES + VBAN_PACKET_MAX_SAMPLES*2;


    struct VBanHeader {
        preamble : [u8; 4],
        sample_rate : u8,
        num_samples : u8,

        // number of channels, where 0 = one channel
        num_channels : u8,
        sample_format : u8,
        stream_name : [u8;16],
        nu_frame : u32
    }

    impl From<[u8; 28]> for VBanHeader {
        fn from (item: [u8; 28]) -> Self {

            // let frame_count : u32 = item[24] as u32 + (item[25] as u32) << 8 + (item[26] as u32) << 16 + (item[27] as u32) << 24;
            let frame_count  = 0;

            Self {
                preamble : item[0..4].try_into().unwrap(),
                sample_rate : item[4],
                num_samples : item[5],
                num_channels : item[6],
                sample_format : item[7],
                stream_name : [item[8], item[9], item[10], item[11], item[12], item[13], item[14], item[15], item[16], item[17], item[18], item[19], item[20], item[21], item[22], item[23]],
                nu_frame : frame_count
            }
        }
    }

    // VBan struct missing

    const VBAN_SR_MASK : u8 = 0x1F;
    const VBAN_SR_MAXNUMBER : u8 = 21;
    const VBAN_SRLIST : [u32; 21] = [
        6000, 12000, 24000, 48000, 96000, 192000, 384000,
        8000, 16000, 32000, 64000, 128000, 256000, 512000,
        11025, 22050, 44100, 88200, 176400, 352800, 705600
    ];

    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum VBanSampleRates {
        SampleRate6000Hz,
        SampleRate12000Hz,
        SampleRate24000Hz,
        SampleRate48000Hz,
        SampleRate96000Hz,
        SampleRate192000Hz,
        SampleRate384000Hz,
        SampleRate8000Hz,
        SampleRate16000Hz,
        SampleRate32000Hz,
        SampleRate64000Hz,
        SampleRate128000Hz,
        SampleRate256000Hz,
        SampleRate512000Hz,
        SampleRate11025Hz,
        SampleRate22050Hz,
        SampleRate44100Hz,
        SampleRate88200Hz,
        SampleRate176400Hz,
        SampleRate352800Hz,
        SampleRate705600Hz
    }

    impl std::fmt::Display for VBanSampleRates {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                VBanSampleRates::SampleRate6000Hz => write!(f, "{} Hz", 6000), 
                VBanSampleRates::SampleRate12000Hz => write!(f, "{} Hz", 12000), 
                VBanSampleRates::SampleRate24000Hz => write!(f, "{} Hz", 24000), 
                VBanSampleRates::SampleRate48000Hz => write!(f, "{} Hz", 48000), 
                VBanSampleRates::SampleRate96000Hz => write!(f, "{} Hz", 96000), 
                VBanSampleRates::SampleRate192000Hz => write!(f, "{} Hz", 192000), 
                VBanSampleRates::SampleRate384000Hz => write!(f, "{} Hz", 384000), 
                VBanSampleRates::SampleRate8000Hz => write!(f, "{} Hz", 8000), 
                VBanSampleRates::SampleRate16000Hz => write!(f, "{} Hz", 16000), 
                VBanSampleRates::SampleRate32000Hz => write!(f, "{} Hz", 32000), 
                VBanSampleRates::SampleRate64000Hz => write!(f, "{} Hz", 64000), 
                VBanSampleRates::SampleRate128000Hz => write!(f, "{} Hz", 128000), 
                VBanSampleRates::SampleRate256000Hz => write!(f, "{} Hz", 256000), 
                VBanSampleRates::SampleRate512000Hz => write!(f, "{} Hz", 512000), 
                VBanSampleRates::SampleRate11025Hz => write!(f, "{} Hz", 11025), 
                VBanSampleRates::SampleRate22050Hz => write!(f, "{} Hz", 22050), 
                VBanSampleRates::SampleRate44100Hz => write!(f, "{} Hz", 44100), 
                VBanSampleRates::SampleRate88200Hz => write!(f, "{} Hz", 88200), 
                VBanSampleRates::SampleRate176400Hz => write!(f, "{} Hz", 176400), 
                VBanSampleRates::SampleRate352800Hz => write!(f, "{} Hz", 352800), 
                VBanSampleRates::SampleRate705600Hz => write!(f, "{} Hz", 705600),             
            }
        }
    }

    impl From<u8> for VBanSampleRates {

        fn from(item : u8) -> Self{
            match item & VBAN_SR_MASK {
                0 => VBanSampleRates::SampleRate6000Hz,
                1 => VBanSampleRates::SampleRate12000Hz,
                2 => VBanSampleRates::SampleRate24000Hz,
                3 => VBanSampleRates::SampleRate48000Hz,
                4 => VBanSampleRates::SampleRate96000Hz,
                5 => VBanSampleRates::SampleRate192000Hz,
                6 => VBanSampleRates::SampleRate384000Hz,
                7 => VBanSampleRates::SampleRate8000Hz,
                8 => VBanSampleRates::SampleRate16000Hz,
                9 => VBanSampleRates::SampleRate32000Hz,
                10 => VBanSampleRates::SampleRate64000Hz,
                11 => VBanSampleRates::SampleRate128000Hz,
                12 => VBanSampleRates::SampleRate256000Hz,
                13 => VBanSampleRates::SampleRate512000Hz,
                14 => VBanSampleRates::SampleRate11025Hz,
                15 => VBanSampleRates::SampleRate22050Hz,
                16 => VBanSampleRates::SampleRate44100Hz,
                17 => VBanSampleRates::SampleRate88200Hz,
                18 => VBanSampleRates::SampleRate176400Hz,
                19 => VBanSampleRates::SampleRate352800Hz,
                20 => VBanSampleRates::SampleRate705600Hz,
                _ => panic!("Invalid value for enum VBanSampleRates ({:b})", item)
        }
    }
}

    const VBAN_PROTOCOL_MASK : u8 = 0xE0;

    #[derive(Debug, PartialEq)]
    enum VBanProtocol {
        VbanProtocolAudio         =   0x00,
        VbanProtocolSerial        =   0x20,
        VbanProtocolTxt           =   0x40,
        VbanProtocolService      =   0x60,
        VbanProtocolUndefined1   =   0x80,
        VbanProtocolUndefined2   =   0xA0,
        VbanProtocolUndefined3   =   0xC0,
        VbanProtocolUndefined4   =   0xE0
    }

    impl From<u8> for VBanProtocol {

        fn from(value: u8) -> Self {
            match value & VBAN_PROTOCOL_MASK {
                0x00 => VBanProtocol::VbanProtocolAudio,
                0x20 => VBanProtocol::VbanProtocolSerial,
                0x40 => VBanProtocol::VbanProtocolTxt,
                0x60 => VBanProtocol::VbanProtocolService,
                0x80 => VBanProtocol::VbanProtocolUndefined1,
                0xA0 => VBanProtocol::VbanProtocolUndefined2,
                0xC0 => VBanProtocol::VbanProtocolUndefined3,
                0xE0 => VBanProtocol::VbanProtocolUndefined4,
                _ => panic!("Invalid value for enum VBanProtocol ({:x})", value & VBAN_PROTOCOL_MASK),
            }
        }
    }

    const VBAN_BIT_RESOLUTION_MASK : u8 = 0x07;

    #[derive(Clone, Copy, Debug)]
    enum VBanBitResolution {
        VbanBitfmt8Int = 0,
        VbanBitfmt16Int,
        VbanBitfmt24Int,
        VbanBitfmt32Int,
        VbanBitfmt32Float,
        VbanBitfmt64Float,
        VbanBitfmt12Int,
        VbanBitfmt10Int,
        VbanBitResolutionMax
    }

    impl From<u8> for VBanBitResolution {
        fn from(item : u8) -> Self {
            match  item & VBAN_BIT_RESOLUTION_MASK  {
                0 => VBanBitResolution::VbanBitfmt8Int,
                1 => VBanBitResolution::VbanBitfmt16Int,
                2 => VBanBitResolution::VbanBitfmt24Int,
                3 => VBanBitResolution::VbanBitfmt32Int,
                4 => VBanBitResolution::VbanBitfmt32Float,
                5 => VBanBitResolution::VbanBitfmt64Float,
                6 => VBanBitResolution::VbanBitfmt12Int,
                7 => VBanBitResolution::VbanBitfmt10Int,
                8 => VBanBitResolution::VbanBitResolutionMax,
                _ => panic!("Invalid value for enum VBanBitResolution ({item})"),
            }
        }
    }

    const VBAN_BIT_RESOLUTION_SIZE : [u8; 6] = [ 1, 2, 3, 4, 4, 8, ];

    const VBAN_RESERVED_MASK : u8 = 0x08;
    const VBAN_CODEC_MASK : u8 = 0xF0;

    #[derive(Debug, PartialEq)]
    enum VBanCodec {
        VbanCodecPcm              =   0x00,
        VbanCodecVbca             =   0x10,
        VbanCodecVbcv             =   0x20,
        VbanCodecUndefined3      =   0x30,
        VbanCodecUndefined4      =   0x40,
        VbanCodecUndefined5      =   0x50,
        VbanCodecUndefined6      =   0x60,
        VbanCodecUndefined7      =   0x70,
        VbanCodecUndefined8      =   0x80,
        VbanCodecUndefined9      =   0x90,
        VbanCodecUndefined10     =   0xA0,
        VbanCodecUndefined11     =   0xB0,
        VbanCodecUndefined12     =   0xC0,
        VbanCodecUndefined13     =   0xD0,
        VbanCodecUndefined14     =   0xE0,
        VbanCodecUser             =   0xF0
    }

    impl From<u8> for VBanCodec {
        fn from(value: u8) -> Self {
            match value & VBAN_CODEC_MASK {
                0x00 => VBanCodec::VbanCodecPcm,
                0x10 => VBanCodec::VbanCodecVbca,
                0x20 => VBanCodec::VbanCodecVbcv,
                0x30 => VBanCodec::VbanCodecUndefined3,
                0x40 => VBanCodec::VbanCodecUndefined4,
                0x50 => VBanCodec::VbanCodecUndefined5,
                0x60 => VBanCodec::VbanCodecUndefined6,
                0x70 => VBanCodec::VbanCodecUndefined7,
                0x80 => VBanCodec::VbanCodecUndefined8,
                0x90 => VBanCodec::VbanCodecUndefined9,
                0xA0 => VBanCodec::VbanCodecUndefined10,
                0xB0 => VBanCodec::VbanCodecUndefined11,
                0xC0 => VBanCodec::VbanCodecUndefined12,
                0xD0 => VBanCodec::VbanCodecUndefined13,
                0xE0 => VBanCodec::VbanCodecUndefined14,
                0xF0 => VBanCodec::VbanCodecUser,
                _ => VBanCodec::VbanCodecUser
            }
        }
    }

    #[derive (PartialEq)]
    enum PlayerState {
        Idle,
        Playing,
    }

    pub struct VbanRecipient {

        socket : UdpSocket,

        sample_rate : Option<VBanSampleRates>,

        num_channels : Option<u8>,

        sample_format : Option<VBanBitResolution>,

        stream_name : Option<[u8;16]>,

        nu_frame : u32,

        state : PlayerState,

        timer : Instant,

        sink : Option<AlsaSink>,

        sink_name : String,

        silence : u32,
    }

    impl VbanRecipient {

        pub fn create(ip_addr : IpAddr, port: u16, stream_name : Option<String>, numch : Option<u8>, sample_rate : Option<VBanSampleRates>, sink_name : String, silence : Option<u32>) -> Option<Self> {

            let mut _sn: Option<[u8; 16]> = match stream_name {
                None => None,
                Some(name) => {
                    if name.len() > VBAN_STREAM_NAME_SIZE {
                        dbg!("Stream name exceeds the limit of {} characters", VBAN_STREAM_NAME_SIZE);
                        return None;
                    }
                    let mut _sn: [u8; 16] = [0u8; 16];
                    for (idx, b) in name.bytes().enumerate(){
                        if idx >= VBAN_STREAM_NAME_SIZE {
                            break;
                        }
                        _sn[idx] = b;
                    }
                    Some(_sn)
                }
            };
            
            let to_addr = (ip_addr, port);
            let result  = VbanRecipient{
                socket :  match UdpSocket::bind(to_addr){
                    Ok(sock) => sock,
                    Err(_) => {
                        dbg!("Could not create socket");
                        return None;
                    },
                },
                
                sample_rate : sample_rate,
                
                num_channels : numch,
                
                sample_format : None,
                
                stream_name : _sn,

                nu_frame : 0,
                
                state : PlayerState::Idle,

                timer : Instant::now(),

                sink : None,

                sink_name,

                silence : match silence {
                    None => 0,
                    Some(val) => val,
                }
            };

            result.socket.set_read_timeout(Some(Duration::new(1, 0))).expect("Could not set timeout of socket");

            println!("VBAN recepipient ready. Waiting for incoming audio packets...");
            Some(result)
        }
        

        pub fn handle(&mut self){
            let mut buf :[u8; VBAN_PACKET_MAX_LEN_BYTES] = [0; VBAN_PACKET_MAX_LEN_BYTES];
            let packet = self.socket.recv_from(&mut buf);
            // let buf = Vec::from(buf);

            if self.state == PlayerState::Playing && self.timer.elapsed().as_secs() > 2 {
                self.state = PlayerState::Idle;
               
                match &self.sink{
                    None => println!("Something's wrong. Expected to find a pcm but it is unitialized."),
                    Some(sink) => {
                        match sink.pcm.drain(){
                            Err(errno) => println!("Error while draining pcm: {errno}"),
                            Ok(()) => (),
                        }
                        match sink.pcm.drop(){
                            Err(errno) => println!("Error while closing pcm: {errno}"),
                            Ok(()) => (),
                        }
                        self.sink = None;
                    }
                }
                println!("idle");
            }

            let size = match packet {
                Ok((size, _addr)) => {
                    size
                },
                _ => return,
            };

            if buf[..4] == *b"VBAN" {

                let head : [u8; 28] = buf[0..28].try_into().unwrap();
                let head = VBanHeader::from(head);

                self.sample_format = Some(head.sample_format.into());
                
                let num_samples = head.num_samples + 1;
                let bits_per_sample = VBAN_BIT_RESOLUTION_SIZE[self.sample_format.unwrap() as usize];
                let codec = VBanCodec::from(head.sample_format);
                let protocol = VBanProtocol::from(head.sample_rate);
                let name_incoming : &str = from_utf8(&head.stream_name).unwrap();
                
                // println!("DEBUG: bps={bits_per_sample}, codec={:?}", codec);
                
                if protocol != VBanProtocol::VbanProtocolAudio {
                    println!("Discarding packet with protocol {:?} because it is not supported.", protocol);
                    return;
                }
                if codec != VBanCodec::VbanCodecPcm {
                    println!("Any codecs other than PCM are not supported (found {:?}).", codec);
                    return;
                }
                if bits_per_sample != 2{
                    println!("Bitwidth other than 16 bits not supported (found {}).", bits_per_sample * 8);
                    return;
                }
                
                let sr : VBanSampleRates  = head.sample_rate.into();
                self.num_channels = Some(head.num_channels + 1);

                match self.stream_name {
                    None => (),
                    Some(name) => {
                        if from_utf8(&name).unwrap() != name_incoming {
                            println!("Discarding packet because stream names don't match.");
                            return;
                        }
                    }
                }

                let audio_data : Vec<u8> = Vec::from(&buf[VBAN_PACKET_HEADER_BYTES + VBAN_PACKET_COUNTER_BYTES..size]);
                let mut to_sink = vec![0; audio_data.len() / bits_per_sample as usize];

                let mut left : i16 = 0;
                let mut right : i16 = 0;
                for (idx, _smp) in audio_data.iter().enumerate() {
                    if idx % 2 == 1 {
                        continue;
                    }

                    let amplitude_le = LittleEndian::read_i16(&audio_data[idx..idx+2]);

                    if idx % 4 == 0 {
                        if amplitude_le > left {
                            left = amplitude_le;
                        }
                    } else {
                        if amplitude_le > right {
                            right = amplitude_le;
                        }
                    }

                    to_sink[idx / 2] = amplitude_le;
                }


                self.timer = Instant::now();
                if self.state == PlayerState::Idle {
                    match &self.sink {
                        Some(_sink) => println!("Something's wrong. Sink is Some() although it should be None"),
                        None => {
                            self.sample_rate = Some(sr);
                            self.sink = Some(AlsaSink::init(&self.sink_name, Some(self.num_channels() as u32), Some(self.sample_rate())).expect("Could not create audio device with the required specs."));

                            println!("Connected to stream {}: \nSR: {} \t Ch: {} \t BPS: {}\n", name_incoming, self.sample_rate(), self.num_channels(), self.bits_per_sample());

                            /* Push silence before the data */
                            let one_ms_smp = self.sample_rate() / 1000;
                            let silence_buf = vec![0i16; (self.sample_rate() / 1000 * self.silence) as usize];
                            self.sink.as_mut().unwrap().write(&silence_buf);
                        }
                    }
                    self.state = PlayerState::Playing;
                } else {
                    if sr != self.sample_rate.unwrap(){
                        println!("SR: {} -> {}", self.sample_rate.unwrap(), sr);
                        self.sample_rate = Some(sr);
                        let sink = self.sink.as_mut().unwrap();
                        sink.pcm.drain();
                        self.sink = Some(AlsaSink::init(&self.sink_name, Some(self.num_channels() as u32), Some(self.sample_rate())).expect("Could not create audio device with the required specs."));
                    }
                }
                let sink = self.sink.as_mut().unwrap();
                sink.write(&to_sink);
                println!("\x1B[1ALeft {:.4}, Right {:.4} (from {num_samples} samples)", (left as f32 / i16::MAX as f32), (right as f32 / i16::MAX as f32));
            } else{
                println!("Packet is not VBAN");
            }
        }


        // GETTER
        fn name(&self) -> Option<[u8;16]>{
            self.stream_name
        }
 
        fn name_str(&self) -> String{
            match &self.stream_name {
                None => String::from(""),
                Some(name) => String::from(from_utf8(name).unwrap())
            }
        }

        fn sample_rate(&self) -> u32 {
            VBAN_SRLIST[self.sample_rate.unwrap() as usize]
        }

        fn bits_per_sample(&self) -> u8 {
            self.sample_format.unwrap() as u8 + 1
        }

        fn num_channels(&self) -> u8 {
            self.num_channels.unwrap() as u8
        }


    }


    pub trait VbanSink {
        fn write(&self, buf : &[i16]);
    }

    // ALSA SINK

    pub struct AlsaSink {
        pcm : PCM,
    }

    impl AlsaSink {

        pub fn init(device : &str, num_channels : Option<u32>, sample_rate : Option<u32>) -> Option<Self> {

            let sink = Self {
                pcm : {
                    PCM::new(device, Direction::Playback, false).expect("Could not create PCM.")
                },
            };

            let num_channels = match num_channels {
                None => {2},
                Some(ch) => ch,
            };
            let rate = match sample_rate {
                None => 44100,
                Some(r) => r,
            };

            {
                let hwp = HwParams::any(&sink.pcm).expect("Could not get hwp.");

                hwp.set_channels(num_channels).expect("Could not set channel number.");
                hwp.set_rate(rate, ValueOr::Nearest).expect("Could not set sample rate.");
                hwp.set_format(Format::s16()).expect("Could not set sample format.");
                hwp.set_access(Access::RWInterleaved).expect("Could not set access.");
                sink.pcm.hw_params(&hwp).expect("Could not attach hwp to PCM.");
            }

            match sink.pcm.start(){
                Ok(()) => (),
                Err(errno) => {
                    println!("Error: {errno}");
                    sink.pcm.drain().expect("Drain failed");
                    match sink.pcm.recover(errno.errno(), true){
                        Ok(()) => (),
                        Err(errno) => println!("Recovering after failed start failed too."),
                    }
                },
            }

            // Debug
            // let ff = pcm.hw_params_current().and_then(|h| h.get_format())?;

            // {
            //     let params = sink.pcm.hw_params_current().unwrap();
            //     println!("(Debug) HwParams: {:?}", params);
            //     let sr = params.get_rate().unwrap();
            //     let nch = params.get_channels().unwrap();
            //     let fmt = params.get_format().unwrap();
            //     let bsize = params.get_buffer_size().unwrap();
            //     let psize = params.get_period_size().unwrap();
                
            //     println!("Created playback device with sr={sr}, channels={nch}, format={fmt}, period size={psize} and buffer size={bsize}.\n");
            // }

            {
                let swp = sink.pcm.sw_params_current().unwrap();
                match swp.set_start_threshold(512) {
                    Ok(()) => (),
                    Err(errno) => println!("Could not set start_threshold sw parameter (error {errno})."),
                }

                let thr = swp.get_start_threshold().unwrap();
                // todo? set silence threshold?
                println!("Start threshold is {thr}.");
            }
            Some(sink)
        }
    
    }

    impl VbanSink for AlsaSink {

        fn write(&self, buf : &[i16]){
            let io = self.pcm.io_i16().unwrap();

            match io.writei(buf){
                Err(errno) => {
                    // Maybe try to investigate the pcm device here and try to reopen it (because broken pipe)

                    println!("Write did not work. Error: {errno}");
                    // let state = self.pcm.state();

                    match self.pcm.recover(errno.errno(), true){
                        Ok(()) => {
                            println!("Was able to recover from error");
                            match io.writei(buf){
                                Ok(_) => (),
                                Err(errno) => println!("Second attempt to write buffer failed ({errno})."),
                            }
                        },
                        Err(errno2) => println!("Could not recover from error (errno2={errno2}"),
                    }
                },
                Ok(_size) => (),
            }

        }
    }

}
