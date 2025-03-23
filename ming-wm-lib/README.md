Library for building [ming-wm](https://github.com/stjet/ming-wm) windows in Rust.

Documentation [here](https://docs.rs/ming-wm-lib).

The most likely usage of this crate is to implement the [WindowLike trait](https://docs.rs/ming-wm-lib/latest/ming_wm_lib/window_manager_types/trait.WindowLike.html), and then set up [IPC](https://docs.rs/ming-wm-lib/latest/ming_wm_lib/ipc/fn.listen.html) (which automatically handles serialisation).

