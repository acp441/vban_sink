# VBAN Sink

written by a Rust newbie - feedback appreciated!

__Work in progress.. but mostly works!__

A sink for VBAN streams for Linux systems using ALSA (works with pipewire too). 

I developed this application for my Raspberry Pi to running Moode Audio to support VBAN. So I can confirm this runs on a RPi 4 with 4 GB RAM. 

# Usage

Start a VBAN stream, for example by using the Voicemeeter application from the creator of VBAN (vb-audio.com). Direct the outgoing stream to the machine that should run vban_sink. Run `vban_sink` (simple as that). Make sure port 6980 is open for incoming udp packets. vban_sink adapts to the incoming sample rate. __Only 16 bit format supported, though!.__

## Options

- -p : Specify a different port (other that 6980)
- -c : Work in progress - not supported yet. 
- -s : Specify a stream name if you only want to accept one specific stream. 
- -x : Prepend silence when starting playback. This is useful to avoid buffer underrun on instable networks.
- -d : Audio device name to be used as sink. Default is 'default' which usually points to the default audio device when using ALSA.
- -m : Execute a script on playback state change.


### Executing a script on playback state change

If the option `-m` is used a script may be executed on playback state change. The script will be invoked with the argmuents "playback_started" or "playback_stopped" respectiely. 