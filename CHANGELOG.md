# 0.1.8

- reduce the amount of boilerplate in sound.rs with the `delegate!()` macro for generating `SoundWrapper`'s methods (thanks [@Microwonk](https://github.com/Microwonk)!) https://github.com/zeozeozeo/kittyaudio/pull/2

# 0.1.7

- fix `Sound` skipping the first three frames (https://github.com/zeozeozeo/kittyaudio/pull/1, thanks [@Sytronic](https://github.com/Sytronik)!)
- add `pause()`, `paused()` and `resume()` methods for `Sound`

# 0.1.4

- Improve perfomance by using [parking_lot](https://github.com/Amanieu/parking_lot) instead of OS mutexes. See https://webkit.org/blog/6161/locking-in-webkit/.
