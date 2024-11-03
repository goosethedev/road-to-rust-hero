use std::{io, net::UdpSocket, time::Duration};

use byteorder::{BigEndian, ReadBytesExt};
use chrono::{DateTime, TimeZone, Timelike, Utc};

/// 4 messages of 12 bytes (metadata, T2, T3, 4 bytes each (u32))
/// Note: T1 should come second, but it's not really needed
const NTP_MESSAGE_LENGTH: usize = 48;
const NTP_TO_UNIX_SECONDS: i64 = 2_208_988_800;

/// Default port for NTP
const LOCAL_ADDR: &'static str = "0.0.0.0:12300";

/// Single timestamp of 4 bytes of seconds and 4 bytes for microseconds
struct NtpTimestamp {
    seconds: u32,
    fraction: u32,
}

// Allow conversions for NtpTimestamp and DateTime<Utc>
impl From<NtpTimestamp> for DateTime<Utc> {
    fn from(ntp_timestamp: NtpTimestamp) -> Self {
        let secs = ntp_timestamp.seconds as i64 - NTP_TO_UNIX_SECONDS;
        let mut nsecs = ntp_timestamp.fraction as f64;
        nsecs *= 1e9;
        nsecs /= (2_f64).powi(32);

        // TODO: Better handle this unwrap
        Utc.timestamp_opt(secs, nsecs as u32).unwrap()
    }
}

impl From<DateTime<Utc>> for NtpTimestamp {
    fn from(datetime: DateTime<Utc>) -> Self {
        let seconds = datetime.timestamp() + NTP_TO_UNIX_SECONDS;
        let mut fraction = datetime.nanosecond() as f64;
        fraction *= (2_f64).powi(32);
        fraction /= 1e9;

        NtpTimestamp {
            seconds: seconds as u32,
            fraction: fraction as u32,
        }
    }
}

/// Struct to keep the entire request process timestamps
struct NtpResult {
    t1: DateTime<Utc>,
    t2: DateTime<Utc>,
    t3: DateTime<Utc>,
    t4: DateTime<Utc>,
}

impl NtpResult {
    fn delay(&self) -> i64 {
        let duration = (self.t4 - self.t1) - (self.t3 - self.t2);
        duration.num_milliseconds()
    }

    fn offset(&self) -> i64 {
        self.delay().abs() / 2
    }
}

struct NtpMessage {
    data: [u8; NTP_MESSAGE_LENGTH],
}

impl NtpMessage {
    /// Empty NtpMessage
    fn new() -> Self {
        Self {
            data: [0; NTP_MESSAGE_LENGTH],
        }
    }

    /// Set request message with metadata
    fn client() -> Self {
        // Metadata structure:
        // - leap seconds: 2 bits (none so zero)
        // - protocol version: 3 bits (version 3)
        // - mode: 3 bits (3 = client mode)
        const VERSION: u8 = 0b00_011_00;
        const MODE: u8 = 0b00_000_011;

        let mut msg = NtpMessage::new();

        // Set first byte with metadata
        msg.data[0] |= VERSION;
        msg.data[0] |= MODE;
        msg
    }

    fn parse_timestamp_at(&self, bit: usize) -> io::Result<NtpTimestamp> {
        // Take a slice of the first byte at the position
        let mut reader = &self.data[bit..bit + 8];

        // Read parts
        let seconds = reader.read_u32::<BigEndian>()?;
        let fraction = reader.read_u32::<BigEndian>()?;

        Ok(NtpTimestamp { seconds, fraction })
    }

    /// Get the timestamp from the server received time
    fn rx_timestamp(&self) -> io::Result<NtpTimestamp> {
        self.parse_timestamp_at(32)
    }

    /// Get the timestamp from the server sent time
    fn tx_timestamp(&self) -> io::Result<NtpTimestamp> {
        self.parse_timestamp_at(40)
    }
}

/// Helper function
fn weighted_mean(values: &[f64], weights: &[f64]) -> f64 {
    let mut result = 0.0;
    let mut sum_of_weights = 0.0;

    for (v, w) in values.iter().zip(weights) {
        result += v * w;
        sum_of_weights += w;
    }

    result / sum_of_weights
}

/// Make a complete NTP request to a server
fn ntp_request_roundtrip(server: &str, port: u16) -> io::Result<NtpResult> {
    let destination = format!("{}:{}", server, port);

    let request = NtpMessage::client();
    let mut response = NtpMessage::new();

    // Setup UDP connection to make request
    let udp = UdpSocket::bind(LOCAL_ADDR)?;
    udp.connect(&destination)?;

    // Make request, taking T1 at start and T4 at end
    let t1 = Utc::now();
    udp.send(&request.data)?;
    udp.set_read_timeout(Some(Duration::from_secs(1)))?;
    udp.recv(&mut response.data)?;
    let t4 = Utc::now();

    // Parse T2 and T3 from response
    let t2: DateTime<Utc> = response.rx_timestamp()?.into();
    let t3: DateTime<Utc> = response.tx_timestamp()?.into();

    Ok(NtpResult { t1, t2, t3, t4 })
}

/// Take samples from various NTP servers worldwide
pub fn get_ntp_offset() -> io::Result<f64> {
    const NTP_PORT: u16 = 123;

    // TODO: Check why this always fails with Resource temporarily unavailable
    let servers = [
        "pool.ntp.org",
        "time.nist.gov",
        "time.apple.com",
        "time.euro.apple.com",
        "time.google.com",
        "time2.google.com",
        // "time.windows.com", // has too much difference with the rest
    ];

    let mut results = Vec::with_capacity(servers.len());

    for server in servers {
        print!("{} => ", &server);

        match ntp_request_roundtrip(&server, NTP_PORT) {
            Ok(res) => {
                println!("{} ms away from local time", res.offset());
                results.push(res);
            }
            Err(err) => {
                println!("? [response timed out]");
                dbg!(err);
            }
        }
    }

    // Calculate average offset
    let mut offsets = Vec::with_capacity(results.len());
    let mut weights = Vec::with_capacity(results.len());

    for res in results {
        let offset = res.offset() as f64;
        let delay = res.delay() as f64;

        // Penalize high network delays
        let weight = 1_000_000.0 / (delay * delay);
        if weight.is_finite() {
            offsets.push(offset);
            weights.push(weight);
        }
    }

    Ok(weighted_mean(&offsets, &weights))
}
