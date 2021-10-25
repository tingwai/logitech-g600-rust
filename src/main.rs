use regex::Regex;
use std::fs;
use std::io;
use std::io::Read;
use std::mem;
use std::process::Command;
use std::os::unix::prelude::AsRawFd;
use std::os::unix::io::{FromRawFd, IntoRawFd, RawFd};
use std::slice;

mod config;

// #define EVIOCGRAB    _IOW('E', 0x90, int)	/* Grab/Release device */
const IOW_MAGIC: u8 = b'E';
const IOW_SEQUENCE: u8 = 0x90;
nix::ioctl_write_int!(eviocgrab, IOW_MAGIC, IOW_SEQUENCE);

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct input_events {
    // event 1
    pub time1: timeval,
    pub type1: u16,
    pub code1: u16,
    pub value1: i32,

    // event 2
    pub time2: timeval,
    pub type2: u16,
    pub code2: u16,
    pub value2: i32,

    // event 3
    pub time3: timeval,
    pub type3: u16,
    pub code3: u16,
    pub value3: i32,

    // event 4
    pub time4: timeval,
    pub type4: u16,
    pub code4: u16,
    pub value4: i32,

    // event 5
    pub time5: timeval,
    pub type5: u16,
    pub code5: u16,
    pub value5: i32,

    // event 6
    pub time6: timeval,
    pub type6: u16,
    pub code6: u16,
    pub value6: i32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct timeval {
    pub tv_sec: i64,
    pub tv_usec: i64,
}

const KDIR: &str = "/dev/input/by-id/";
const KPREFIX: &str = "usb-Logitech_Gaming_Mouse_G600_";
const KSUFFIX: &str = "-if01-event-kbd";

fn main() -> Result<(), io::Error>{
    println!("Starting G600 Linux controller.\n");
    println!("It's a good idea to configure G600 with Logitech Gaming Software before running this program:");
    println!(" - assign left, right, middle mouse button and vertical mouse wheel to their normal functions");
    println!(" - assign the G-Shift button to \"G-Shift\"");
    println!(" - assign all other keys (including horizontal mouse wheel) to arbitrary (unique) keyboard keys");
    println!();

    let config = config::read_config()?;

    let path = get_device_path()?;
    let full_path: String = KDIR.to_owned() + &path;
    let file = fs::OpenOptions::new()
        .read(true)
        .open(full_path)?;

    unsafe{
        let err: Result<i32, nix::Error> = eviocgrab(file.as_raw_fd(), 1);
        let _err = match err {
            Ok(err) => err,
            Err(e) => panic!("Error running ioctl eviocgrab macro: {}", e),
        };
    };

    // SAFETY: no other functions should call `from_raw_fd`, so there
    // is only one owner for the file descriptor.
    let raw_fd: RawFd = file.into_raw_fd();
    let mut f = unsafe { fs::File::from_raw_fd(raw_fd) };

    loop {
        // https://stackoverflow.com/a/25411013
        let mut events: input_events = unsafe { mem::zeroed() };
        let events_size = mem::size_of::<input_events>();
        unsafe {
            let mut events_slice = slice::from_raw_parts_mut(
                &mut events as *mut _ as *mut u8,
                events_size
            );
            // read buffer into struct
            // TODO fix unwrap
            f.read(&mut events_slice).unwrap_or(0);
        };

        if events.type1 != 4 { continue }
        if events.code1 != 4 { continue }
        if events.type2 != 1 { continue }

        let pressed = events.value2 != 0;
        let scancode = events.value1 & !0x70000;  // no idea where hex value comes from
        let button = get_button(scancode);


        let wm = get_wm_class();
        let wm = match wm {
            Ok(wm) => wm,
            // might error out when active window is an empty workspace in i3
            Err(_err) => continue,
        };
        let (instance, _class_name) = wm;


        // println!("{:#?}", events);
        // println!("{}  {}", &instance, class_name);
        println!("pressed: {}  scancode: {:#X}  button: {}", pressed, scancode, button);
        // println!("{}", config::command(scancode));
        if pressed {
            match config::run_command(&config, &instance, &button.to_owned()) {
                Ok(_) => {},
                Err(err) => println!("Error: {}", err.to_string()),
            };
        } else {
            config::stop_command(&config, &button.to_owned());
        }

        println!()

    }
}

fn get_button(scancode: i32) -> &'static str {
    match scancode {
        4 => "G_shift_G9",
        5 => "G_shift_G10",
        6 => "G_shift_G11",
        7 => "G_shift_G12",
        8 => "G_shift_G13",
        9 => "G_shift_G14",
        10 => "G_shift_G15",
        11 => "G_shift_G16",
        12 => "G_shift_G17",
        13 => "G_shift_G18",
        14 => "G_shift_G19",
        15 => "G_shift_G20",
        30 => "G9",
        31 => "G10",
        32 => "G11",
        33 => "G12",
        34 => "G13",
        35 => "G14",
        36 => "G15",
        37 => "G16",
        38 => "G17",
        39 => "G18",
        45 => "G19",
        46 => "G20",
        _ => "",
    }
}

fn get_wm_class() -> Result<(String, String), io::Error> {
    // `xdotool getactivewindow` to get window ID
    let output = Command::new("bash").arg("-c")
        .arg("xdotool getactivewindow")
        .output()?;
    if !output.stderr.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            String::from_utf8(output.stderr.clone()).unwrap_or("failed to run xdotool".to_string()),
        ));
    }

    let out = output.stdout.to_owned();
    let window_id = String::from_utf8(out.clone())
        .unwrap_or(format!("failed to convert bytes to utf8: {:?}", out.clone()));

    // `xprop WM_CLASS` to get instance and class name for given window ID
    let output = Command::new("bash").arg("-c")
        .arg(format!("xprop WM_CLASS -id {}", &window_id))
        .output()?;
    if !output.stderr.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            String::from_utf8(output.stderr.clone()).unwrap_or("failed to run xprop".to_string()),
        ));
    }

    let out = output.stdout.to_owned();
    let wm_class = String::from_utf8(out.clone())
        .unwrap_or(format!("failed to convert bytes to utf8: {:?}", out.clone()));

    // split output into instance and class name
    // eg. 'WM_CLASS(STRING) = "Navigator", "Firefox"' --> ["Navigator", "Firefox"]
    let re = Regex::new("\"(.*?)\"").unwrap();
    let instance_and_class: Vec<String> = re.find_iter(&wm_class)
        .map(|word| {
            // remove double quotes surrounding word
            let mut chars = word.as_str().chars();
            chars.next();
            chars.next_back();
            return chars.as_str().to_owned();
        })
        .collect();
    if instance_and_class.len() < 2 {
        // unrecognizable program, but don't return error because just fallback to default keybindings
        return Ok(("".to_owned(), "".to_owned()));
    }

    return Ok((instance_and_class[0].clone(), instance_and_class[1].clone()));
}


fn get_device_path() -> Result<String, io::Error> {
    let paths = fs::read_dir(KDIR)?;

    for path in paths {
        let file_name = path?.file_name();

        let name = file_name.to_str().unwrap_or("");
        if name.starts_with(KPREFIX) && name.ends_with(KSUFFIX) {
            return Ok(name.to_string());
        }
    }

    return Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("Failed to find g600 at path {}/{}...{}", KDIR, KPREFIX, KSUFFIX),
    ))
}
