Though some windows (windows = apps, in ming-wm terminology) are built-in to the ming binary (eg, the help and about windows), most windows are separate binaries. This makes it possible for windows to be written in any language (though it will be far easier to write on in Rust because of the `ming-wm-lib` crate), and ensures that errors / crashes in the windows don't result in the window manager also crashing.

Some good examples are in `src/bin`. Another good example is the [Koxinga web browser](https://github.com/stjet/koxinga). All of these are written in Rust, though examples of windows written in other languages will be created later.

## Window Binary Discovery

The window manager automatically searches for window binaries and adds them to the start menu if:

- The binary is in the same directory as the ming binary.
- The binary file name is in the form `ming<Category>_<App_Name>`. Where `<Category>` is a category name in the start menu. For example, the Terminal binary is named `mingUtils_Terminal`.

## Inter-Process Communication (IPC)

Since the windows and the window manager are separate binaries, they need some way to communicate. This is achieved by the window manager spawning the window binary with piped stdout and piped stdin. See `src/proxy_window_like.rs` and `ming-wm-lib/src/ipc.rs` for more information.

The serialization format is in `ming-wm-lib/src/serialize.rs`. Make sure any newlines (`\n`) in strings are removed before/after serializations. When doing IPC, the window manager assumes the response to a query is one line, so if a newline is present, it will fail to parse the response.

## Hello, World!

A minimal example using `ming-wm-lib`.

`Cargo.toml`:

```toml
[package]
name = "example"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "mingMisc_Example"
path = "src/main.rs"

[dependencies]
ming-wm-lib = "0.1.3"
```

`src/main.rs`:

```rust
use std::vec::Vec;
use std::vec;

use ming_wm_lib::window_manager_types::{ DrawInstructions, WindowLike, WindowLikeType };
use ming_wm_lib::messages::{ WindowMessage, WindowMessageResponse };
use ming_wm_lib::framebuffer_types::Dimensions;
use ming_wm_lib::themes::ThemeInfo;
use ming_wm_lib::ipc::listen;

struct Example {
  //
}

impl WindowLike for Example {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      //placeholder
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    vec![DrawInstructions::Text([2, 2], vec!["nimbus-roman".to_string()], "Hello, World!".to_string(), theme_info.text, theme_info.background, None, None)]
  }
  
  fn title(&self) -> String {
    "Example".to_string()
  }
  
  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::Window
  }
  
  fn ideal_dimensions(&self, _dimensions: Dimensions) -> Dimensions {
    [410, 410]
  }
  
  fn resizable(&self) -> bool {
    true
  }
}

impl Example {
  fn new() -> Self {
    //doesn't do anything atm
    Self {}
  }
}

fn main() {
  listen(Example::new());
}
```

To install:

```bash
cargo build --release
mv target/release/mingMisc_Example /usr/bin/mingMisc_Example #or whatever directory the ming binary is in
```

![The example Hello World window!](/docs/images/window_example.png)

## Tips

- For windows that are separate binaries, the Elm Architecture obviously cannot be enforced (unless the window is written in Rust and uses the `ming-wm-lib`. However, the design of the IPC and the nature of the window manager being keyboard-driven makes it so using the Elm Architecture is highly recommended.
- Since the window manager currently queries and reads the responses to/from window binaries in the main thread, while the response is being waited for, the window manager is "frozen". Therefore, time-consuming tasks (>1 second) should not be done in the main thread, but rather a separate thread. For example, the ming-wm audio player (`src/bin/audio_player.rs`) does the time-consuming process of reading audio files in a separate thread to not hold up the window manager, and provide quick responses.

