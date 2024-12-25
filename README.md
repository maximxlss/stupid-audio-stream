# Stupid audio stream

This thing just streams audio from a WASAPI device to a UDP socket and back in the stupidiest way possible and still works better than ffmpeg (The reason is (probably, idk) that ffmpeg currently only supports direct show and not WASAPI, but whatever)

This uses absolutely zero compression, so only works if you have enough bandwidth for 16 bit 48000 hz PCM, that is, 1.5 mbit/s. I don't think you can use this in any way that is not a direct LAN.

Why did I make this? Because I needed a stupid simple headless way to stream audio from my laptop to my pc, to use the mic and headphones on both devices at once (that's easy with this and additional vb cable stuff).

Should you use this? No, probably not, but you can try.

