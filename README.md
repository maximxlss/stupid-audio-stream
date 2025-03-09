# Stupid audio stream

This thing just streams audio from a WASAPI device to a UDP socket and back in the stupidiest way possible and still works better than ffmpeg (The reason is (probably, idk) that ffmpeg currently only supports direct show and not WASAPI, but whatever)

This uses absolutely zero compression, so only works if you have enough bandwidth for 32 bit 48000 hz PCM, that is, constant 3 mbit/s. I don't think you can use this in any way that is not a direct LAN.

Why did I make this? Because I needed a stupid simple headless way to stream audio from my laptop to my pc, to use the mic and headphones on both devices at once (that's easy with this and additional vb cable stuff).

Should you use this? No, probably not, but you can try.

### 🔥 NEW! 🔥
UDP turned out to be not stable and not stateless enough, so I've implemented an alternative. To use it, just use urls with protocol `idc` like `idc://1.2.3.4:5678` (stands for I Don't Care). This thing will establish a TCP connection when possible (with source as the server). In case of any connection errors, it will silently drop everything, while trying to reconnect. Sounds crazy, right? Sounds perfect to me.

In my testing it actually survives restarting either side and disconnecting the cable, so you can just run it in any order or way you want and forget about it. 

