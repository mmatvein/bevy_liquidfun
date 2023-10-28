### 🚨WARNING: WORK IN PROGRESS🚨
This crate is a work in progress and version changes may introduce breaking changes without warning, even in minor version number releases.

# bevy_liquidfun
A Bevy friendly wrapper of Box2D and LiquidFun.

This crate wraps [libliquidfun-sys](https://github.com/mmatvein/libliquidfun-sys) to integrate Box2D and LiquidFun with the [Bevy game engine](https://github.com/bevyengine/bevy).

### Remarks
- The library is not fully featured yet. It has been developed with 'Example Driven Development' so far, which means that the examples showcase _everything_ available at this moment.
- Feedback on the APIs, usability & overall code quality is very welcome. I am not very accustomed to Rust and it's a learning process.
- Pull Requests are welcome, but this crate is so early in development that it is important for me to have full and thorough understanding of all the bits that go into it.
 
### Acknowledgements
This crate is made possible by excellent prior work by others. Huge thanks go to:

[Box2D](https://github.com/erincatto/box2d) by Erin Catto

[LiquidFun](https://github.com/google/liquidfun) by Google

[autocxx](https://github.com/google/autocxx) by Google

[LiquidFun rebase onto newer Box2D](https://github.com/Birch-san/box2d/tree/liquidfun-rebase) by Birch-san

### License

This work is licensed under either of [Apache License, Version 2.0](https://github.com/mmatvein/libliquidfun-sys/blob/main/LICENSE-APACHE) or [MIT license](https://github.com/mmatvein/libliquidfun-sys/blob/main/LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
