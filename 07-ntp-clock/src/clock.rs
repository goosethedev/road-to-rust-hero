use chrono::{DateTime, Local, TimeZone};
use std::io::Error;

pub struct Clock;

impl Clock {
    /// Get the local datetime
    pub fn get() -> DateTime<Local> {
        Local::now()
    }

    /// Change the system call in a UNIX-like system
    #[cfg(not(target_os = "windows"))]
    pub fn set<Tz: TimeZone>(t: DateTime<Tz>) -> Result<(), Error> {
        use libc::{settimeofday, suseconds_t, time_t, timeval, timezone};

        // Set the local timezone
        let t = t.with_timezone(&Local);

        // Manually alloc a timeval C struct
        let mut u: timeval = unsafe { std::mem::zeroed() };

        // Set the seconds and microseconds to the timeval
        u.tv_sec = t.timestamp() as time_t;
        u.tv_usec = t.timestamp_subsec_micros() as suseconds_t;

        // Store the result
        let ret;

        // Make the C call to change the system time
        unsafe {
            let mock_tz: *const timezone = std::ptr::null();
            ret = settimeofday(&u as *const timeval, mock_tz);
        };

        // Store the last OS error
        let err = Error::last_os_error();

        if ret == 0 {
            Ok(())
        } else {
            Err(err)
        }
    }
}
