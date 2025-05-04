use std::os::fd::RawFd;
use std::mem::size_of;
use std::ffi::CString;

use libc::{ open, close, read, poll, pollfd, input_event, timeval, __u16, __s32, c_void };

//https://stackoverflow.com/questions/15949163/read-from-dev-input#15949311
//https://www.man7.org/linux/man-pages/man2/poll.2.html

#[allow(non_camel_case_types)]
#[repr(u16)]
#[derive(Clone, Copy, PartialEq)]
pub enum EventType {
  EV_SYN = 0, //event sep
  EV_KEY,
  EV_REL,
  EV_ABS,
  //nothing below will probably ever be relevant to ming-wm
  EV_MSC, //misc
  EV_SW, //switch/toggle
  EV_LED,
  EV_SND,
  EV_REP,
  EV_FF,
  EV_PWR,
  EV_FF_STATUS,
  Unknown(__u16),
}

impl TryFrom<__u16> for EventType {
  type Error = ();

  fn try_from(value: __u16) -> Result<Self, Self::Error> {
    //if the list is any longer should probably somehow make this a macro
    let values = [Self::EV_SYN, Self::EV_KEY, Self::EV_REL, Self::EV_ABS, Self::EV_MSC, Self::EV_SW, Self::EV_LED, Self::EV_SND, Self::EV_REP, Self::EV_FF, Self::EV_PWR, Self::EV_FF_STATUS];
    let value = value as usize;
    if value >= values.len() {
      Err(())
    } else {
      Ok(values[value])
    }
  }
}

//we do not care about time. no one cares about time (probably)
pub struct InputEvent {
  pub type_: EventType,
  pub code: __u16, //depends on EventType
  pub value: __s32,
}

pub struct Input(RawFd);

impl Input {
  pub fn new(input_name: &str) -> Result<Self, ()> {
    let input_name = CString::new(input_name).unwrap();
    let fd = unsafe { open(input_name.as_ptr(), libc::O_RDONLY | libc::O_NONBLOCK) };
    if fd == -1 {
      Err(())
    } else {
      Ok(Self(fd))
    }
  }
}

impl Iterator for Input {
  type Item = InputEvent;

  fn next(&mut self) -> Option<Self::Item> {
    //wait until there is something available
    let mut fds = vec![pollfd {
      fd: self.0,
      events: libc::POLLIN, //return when "there is data to read"
      revents: 0,
    }];
    let poll_result = unsafe { poll(fds.as_mut_ptr(), 1, -1) }; //neg num means no timeout
    if poll_result == -1 {
      return None;
    }
    //now read the event
    let ie_size = size_of::<input_event>();
    let mut ie = input_event {
      time: timeval {
        tv_sec: 0,
        tv_usec: 0,
      },
      type_: 0,
      code: 0,
      value: 0,
    };
    let read_result = unsafe { read(self.0, &mut ie as *mut _ as *mut c_void, ie_size) };
    if read_result == -1 {
      return None;
    }
    let type_ = ie.type_.try_into().unwrap_or(EventType::Unknown(ie.type_));
    Some(Self::Item {
      type_,
      code: ie.code,
      value: ie.value,
    })
  }
}

impl Drop for Input {
  fn drop(&mut self) {
    unsafe { close(self.0) };
  }
}
