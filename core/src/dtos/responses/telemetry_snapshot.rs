use serde::{Deserialize, Serialize};
use bitflags::bitflags;

// Sample response of GET /status against the Flight Bridge API
// NOTE: This reflects the DTO TelemetrySnapshot
// {
//   "seq": 4215,
//   "as_of_ms": 1734639123456,
//   "status": {
//     "armed": true,
//     "failsafe": false,
//     "flight_mode_bits": 64,
//     "arming_flags_bits": 1,
//     "disable_reasons_bits": 0,
//     "loop_time_us": 250
//   },
//   "battery": { "voltage_v": 3.71, "current_a": 4.2, "mah_drawn": 112, "low": false, "critical": false },
//   "attitude": { "roll_deg": 2.3, "pitch_deg": -1.1, "yaw_deg": 187.5 },
//   "rc": { "roll": 1500, "pitch": 1500, "yaw": 1500, "throttle": 1200, "aux": [1000, 2000] },
//   "altitude": { "baro_cm": 123, "v_speed_cms": -3 },
//   "gps": null,
//   "link": { "rssi": 725, "lq": 95, "protocol": "CRSF" },
//   "last_event": { "kind": "ModeChanged", "ts_ms": 1734639119000, "data": {"flight_mode_bits": 64} }
// }


// ---------- bitflags ----------
bitflags! {
    #[derive(Default)]
    pub struct FlightModeFlags: u32 {
        const ANGLE   = 1 << 0;
        const HORIZON = 1 << 1;
        const ACRO    = 1 << 6;
    }
}
bitflags! {
    #[derive(Default)]
    pub struct ArmingFlags: u32 {
        const OK_TO_ARM   = 1 << 0;
        const MSP_ACTIVE  = 1 << 2;
    }
}
bitflags! {
    #[derive(Default)]
    pub struct DisableReasonsFlags: u32 {
        const THROTTLE   = 1 << 0;
        const MSP        = 1 << 1;
        const BATTERY    = 1 << 2;
    }
}

// ---------- serde helpers for bitflags<u32> ----------
mod ser_bits_u32 {
    use super::*;
    use serde::{Deserializer, Serializer};
    pub fn ser_mode<S>(f: &FlightModeFlags, s: S) -> Result<S::Ok, S::Error>
    where S: Serializer { s.serialize_u32(f.bits()) }
    pub fn de_mode<'de, D>(d: D) -> Result<FlightModeFlags, D::Error>
    where D: Deserializer<'de> {
        let bits = u32::deserialize(d)?;
        Ok(FlightModeFlags::from_bits_truncate(bits))
    }
    pub fn ser_arm<S>(f: &ArmingFlags, s: S) -> Result<S::Ok, S::Error>
    where S: Serializer { s.serialize_u32(f.bits()) }
    pub fn de_arm<'de, D>(d: D) -> Result<ArmingFlags, D::Error>
    where D: Deserializer<'de> {
        let bits = u32::deserialize(d)?;
        Ok(ArmingFlags::from_bits_truncate(bits))
    }
    pub fn ser_dis<S>(f: &DisableReasonsFlags, s: S) -> Result<S::Ok, S::Error>
    where S: Serializer { s.serialize_u32(f.bits()) }
    pub fn de_dis<'de, D>(d: D) -> Result<DisableReasonsFlags, D::Error>
    where D: Deserializer<'de> {
        let bits = u32::deserialize(d)?;
        Ok(DisableReasonsFlags::from_bits_truncate(bits))
    }
}

// TODO: This reflects the JSON from the API Flight Bridge service at `/status`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    #[serde(rename = "as_of_ms", alias = "ts_ms")]
    pub as_of_ms: u64,                  // was ts_ms; wire name is as_of_ms
    #[serde(default)]
    pub seq: u64,                       // monotonic sequence for events/snapshots
    pub status: FcStatus,
    pub rc: Option<RcChannels>,
    pub attitude: Option<Attitude>,
    pub rates: Option<Rates>,
    pub battery: Option<Battery>,
    pub altitude: Option<Altitude>,
    pub gps: Option<Gps>,
    pub link: Option<Link>,
    pub cpu_load: Option<u8>,
    #[serde(default)]
    pub last_event: Option<TelemetryEvent>, // optional convenience
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcStatus {
    pub armed: bool,
    pub failsafe: bool,

    // JSON fields are *_bits; internally you still get bitflags.
    #[serde(
        rename = "flight_mode_bits",
        serialize_with = "ser_bits_u32::ser_mode",
        deserialize_with = "ser_bits_u32::de_mode"
    )]
    pub flight_mode: FlightModeFlags,

    #[serde(
        rename = "arming_flags_bits",
        serialize_with = "ser_bits_u32::ser_arm",
        deserialize_with = "ser_bits_u32::de_arm"
    )]
    pub arming_flags: ArmingFlags,

    #[serde(
        rename = "disable_reasons_bits",
        default,
        serialize_with = "ser_bits_u32::ser_dis",
        deserialize_with = "ser_bits_u32::de_dis"
    )]
    pub disable_reasons: DisableReasonsFlags,

    pub loop_time_us: Option<u32>,
}

// ---- the rest of your structs stay the same ----
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RcChannels {
    pub roll: u16, pub pitch: u16, pub yaw: u16, pub throttle: u16,
    #[serde(default)] pub aux: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attitude { pub roll_deg: f32, pub pitch_deg: f32, pub yaw_deg: f32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rates { pub gyro_dps: Vec3, pub accel_g: Vec3 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Battery {
    pub voltage_v: f32,
    pub current_a: Option<f32>,
    pub mah_drawn: Option<u32>,
    pub cells: Option<u8>,
    pub low: bool,
    pub critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Altitude { pub baro_cm: Option<i32>, pub v_speed_cms: Option<i32> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gps {
    pub fix: bool, pub sats: u8, pub lat_e7: i32, pub lon_e7: i32,
    pub alt_cm: i32, pub ground_speed_cms: u32, pub ground_course_deg: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub rssi: Option<u16>, pub lq: Option<u8>, pub protocol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TelemetryEvent {
    Armed { ts_ms: u64 },
    Disarmed { ts_ms: u64 },
    ModeChanged { ts_ms: u64, flight_mode_bits: u32 },
    BatteryLow { ts_ms: u64, voltage_v: f32 },
    Failsafe { ts_ms: u64, active: bool },
}
