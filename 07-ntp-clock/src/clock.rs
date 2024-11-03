use chrono::{DateTime, Duration, Local, TimeZone, Utc};
use std::io::Error;

use crate::ntp::get_ntp_offset;

pub struct Clock;

impl Clock {
    /// Get the local datetime
    pub fn get_local() -> DateTime<Local> {
        Local::now()
    }

    /// Get the datetime by consulting a series of NTP servers
    pub fn get_from_ntp() -> DateTime<Local> {
        let offset = get_ntp_offset().expect("Error getting NTP offset");

        // Make sure the offset is less than 200 ms
        let adjust_ms = offset.signum() * offset.abs().min(200.0);
        let adjust_ms = Duration::milliseconds(adjust_ms as i64);

        let now = Utc::now() + adjust_ms;
        now.with_timezone(&Local)
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
