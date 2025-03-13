> This is not extensive technical documentation of the project, but a technical overview.
> If reading this in the "About" app inside ming-wm, use the 'j' and 'k' keys to scroll.
> Recommended reading music: Hegira by such

Ming-wm, as the name implies, is great at liberating the nation of Mongol hordes, and great at collapsing when invaded by Jurchens. It is licensed under the GPLv3.

Also, it is a window manager. The window manager manages "window-likes":

```rust
pub enum WindowLikeType {
  LockScreen,
  Window,
  DesktopBackground,
  Taskbar,
  StartMenu,
  WorkspaceIndicator,
}
```

All of these are called "window-likes" because to the window manager they are all essentially all the same; they all implement the same trait.

```rust
pub trait WindowLike {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse;

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions>;

  //properties
  fn title(&self) -> String {
    String::new()
  }

  fn resizable(&self) -> bool {
    false
  }

  fn subtype(&self) -> WindowLikeType;

  fn ideal_dimensions(&self, dimensions: Dimensions) -> Dimensions; //needs &self or its not object safe or some bullcrap
}
```

The only thing special about `Window` window-likes is that the window manager draws window decorations for it automatically. Well, I don't want to lie. There are a few other special things but that's the main one.

## The Event Loop

The event loop goes like this:

1. Keyboard event received, sent to the window manager
2. The window manager interprets it. It could be a shortcut to say, open the start meny. Or, if a window-like is currently focused, the window manager will probably forward the keyboard event to that window-like by calling the window-like's `handle_message` method
3. (Only if sent to a window-like) The window-like receives and processes it. It returns a `WindowMessageResponse`:
> ```rust
> pub enum WindowMessageResponse {
>   Request(WindowManagerRequest),
>   JustRerender,
>   DoNothing,
> }
> ```
4. If the window manager decides the keyboard event means some or all of the screen needs to be redrawn (eg, it was a valid shortcut, or the window-like it sent the event to returned something that wasn't a `DoNothing`), it will go and get the drawing instructions from all the window-likes that need to be redrawn by looping through them and calling their `draw` method

Nothing except key presses trigger redraws. That means no mouse and no animations. This is a positive. This is a positive. I truly believe that. This is a positive. Having a window manager and windows that don't require taking hands off the keyboard (or rather, entirely designed to be keyboard operated) makes using it very fast and efficient, with no pain of not having a mice and needing to use a shitty mousepad. Videos are nice, but animations and the like are annoying and have no place in a good window manager.

## Drawing / Rendering

For each window-like it (re)draws, it creates a new framebuffer, then draws to the framebuffer, the draw instructions it received from the window-like. Then, it draws that new framebuffer onto the actual linux framebuffer (ie, draws the window-like to the screen). So, there is no issue with window-likes overlapping.

One may wonder why exactly the window manager receives drawing instructions from the window-likes and does the actual drawing of the window-likes. Why not just receive the raw pixels of the window-like from the window-like, and just handle compositing the window-likes? Receiving draw instructions does have the advantage of having a significantly better best-case scenario when transferring data between the window-like and window manager, as letting the window-like do the drawing and handing the result to the window manager will mean always transferring `height*width*bytes per pixel` bytes. Additionally, draw instructions are significantly more readably and easier to debug. Finally, asking window-likes to do their own drawing results in window-likes being required to contain lots of drawing code (especially text drawing code!!), rather than just telling the window manager to draw rectangles, lines, and text. While all the window-likes and windows in this repo all rely on the same `framebuffer.rs`, if someone were to write a window in say, Lisp Scheme, they would need to write all that logic again. But the real answer is because that is how it was written in ming-os, from which much of the core code comes from, and it works well, though the lack of a instruction length upper-bound is concerning and potentially inefficient. I don't feel like rewriting it and I don't believe it will become a problem.

## Non-window Window-likes

Recall that the `WindowLikeType`s were LockScreen, DesktopBackground, Taskbar, StartMenu and WorkspaceIndicator (ignoring Window). What they should do is fairly self-explanatory, but as a brief overview:

- Lock screen: Initial state of the window manager. Is the only window-like until the correct password is entered
- Desktop background: Displays the desktop background, behind the windows. Can be a solid colour, or a .bmp image
- Taskbar: Shows currently open windows in the current workspace, and manages the start menu
- Start menu: Shows the window (app) categories, and opens the window (apps) requested
- Workspace indicator: Shows which workspace the window manager is currently in. There are 9 workspaces, each of which can contain their own set of windows. The workspaces can be switched to or out of easily, and windows can be moved easily between them

Each of these receives special, privileged messages from the window manager, in order to be useful and function. For example, the taskbar is notified whenever a window is opened or closed, and the workspace indicator receives a message whenever the workspace is changed.

In some cases, these non-window window-likes also get special rights. For example, only the start menu and taskbar can open window(-likes). The start menu for obvious reasons (opening the windows it was asked to open), and the taskbar needs it to open the start menu. In the future, the taskbar and window manager may be rewritten so the window manager, instead of the taskbar, opens the start menu. Other examples are only the lock screen being able to unlock (if the password is correct), and only the start menu being able to lock (if the lock option is executed).

All of these non-window window-likes are compiled into the window manager binary. They are not separate binaries/processes as they aren't really expected to be modified or swapped out, and are "essential" to the function of the window manager.

## Window Window-likes

Also known as apps. "Apps" and "windows" will be used mostly interchangeably, with the important nuance that there can only be multiple windows opened of a single app.

Windows can be moved and resized over the desktop background (they cannot overlap with the workspace indicator or taskbar, or go off the screen). They also come with a window decoration on the top displaying the title of the window.

As windows are mostly just like other window-likes, they can be compiled in as part of the window manager binary. However, most apps in this repo are separate binaries (see `src/bin`). They use `proxy_window_like.rs` which implements the `WindowLike` trait and proxies talking to the window binary. The window is a child process. Messages are sent to it through piped in stdin, and responses are received through stdout. These apps aren't terribly fancy but the performance impact seems to be unnoticable.

As apps are just any old binary that support the IPC that `proxy_window_like.rs` does, they can be written in any language, and be completely separate from this project. Of course, if they are not written in Rust, extra code is needed to do that IPC.

Binaries in the same directory as the window manager binary and following a specific name format are automatically added to the start menu. See `docs/system/writing_windows.md` for more information.

Some of the apps included are Malvim, the subset of vim (a text editor) I use, Minesweeper, and an Audio Player.

## More on Window-likes

Further documentation on specific window-likes can be found in `docs/window-likes`.

## Themeing

The window manager passes information about it's theme to all window-likes as a parameter to `draw`, so windows can have appropriate background colours, highlight colours, text colours, etc.

//can't change themes yet. in fact, no other themes yet

## Fonts / Text

//
//Japanese / Chinese characters can only be used for display, not input, as there is no CJK input system. yet. And these text inputs don't yet handle multi-byte input very gracefully
