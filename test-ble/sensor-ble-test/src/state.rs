//! Shared LED state — one `LedState` per LED, accessed concurrently by the BLE
//! GATT-write handler and the per-LED blink task.

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

pub const DEFAULT_PERIOD_MS: u32 = 500;
pub const MIN_PERIOD_MS: u32 = 20;
pub const MAX_PERIOD_MS: u32 = 60_000;

pub const PAYLOAD_LEN: usize = 5;
pub type Payload = [u8; PAYLOAD_LEN];

pub struct LedState {
    enabled:   AtomicBool,
    period_ms: AtomicU32,
    signal:    Signal<CriticalSectionRawMutex, ()>,
}

impl LedState {
    pub const fn new() -> Self {
        Self {
            enabled:   AtomicBool::new(false),
            period_ms: AtomicU32::new(DEFAULT_PERIOD_MS),
            signal:    Signal::new(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    pub fn period_ms(&self) -> u32 {
        self.period_ms.load(Ordering::Acquire)
    }

    pub fn set(&self, enabled: bool, period_ms: u32) {
        let clamped = period_ms.clamp(MIN_PERIOD_MS, MAX_PERIOD_MS);
        self.enabled.store(enabled, Ordering::Release);
        self.period_ms.store(clamped, Ordering::Release);
        self.signal.signal(());
    }

    pub async fn wait_change(&self) {
        self.signal.wait().await;
    }

    pub fn encode(&self) -> Payload {
        let mut buf = [0u8; PAYLOAD_LEN];
        buf[0] = self.enabled() as u8;
        buf[1..5].copy_from_slice(&self.period_ms().to_le_bytes());
        buf
    }

    pub fn decode(buf: &Payload) -> Option<(bool, u32)> {
        let enabled = match buf[0] {
            0 => false,
            1 => true,
            _ => return None,
        };
        let period = u32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]);
        Some((enabled, period))
    }
}

pub static RED:   LedState = LedState::new();
pub static GREEN: LedState = LedState::new();
pub static BLUE:  LedState = LedState::new();
