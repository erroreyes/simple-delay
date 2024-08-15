# Simple-Delay

This is a simple delay plugin made in Rust and the nih-plugs framework. It is an attempt at learning about delay lines, ring buffers, and other standard audio processes that involve delay units. 

It started as a simple delay, per its name, but as an effort to figure out interpolation, I added a few badly coded things that make it sound really bad, but in a good way in my opinion. That said, I like noisy effects, and this thing can get noisy.

With that in mind, don't expect this to be an amazing delay effect, but it is still fun to tweak around, especially when freezing the buffer and changing the "interpolation modes". There is a "Pitch" parameter that was originally an attempt at figuring out pitch shifting, but I abandoned that, and it's left in there because...

If you make it in here and check this out, please, please, feel free to check out the code and to share ways that this can improve. I love learning!!

## Building

After installing [Rust](https://rustup.rs/), you can compile Simple-Delay as follows:

```shell
cargo xtask bundle simple-delay --release
```
