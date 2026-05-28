//! SoftDevice configuration, GATT server definition, and the advertising /
//! connection / event loop.

use core::mem;

use nrf_softdevice::ble::advertisement_builder::{
    Flag, LegacyAdvertisementBuilder, LegacyAdvertisementPayload, ServiceList,
};
use nrf_softdevice::ble::{gatt_server, peripheral};
use nrf_softdevice::{raw, Softdevice};

use crate::state::{self, LedState, Payload};

const DEVICE_NAME: &[u8] = b"XiaoLED";

/// 128-bit service UUID, little-endian on the wire.
/// Human form: `a8c1f8a0-0001-4b1a-8f7e-000000000000`
const LED_SERVICE_UUID_LE: [u8; 16] =
    0xa8c1_f8a0_0001_4b1a_8f7e_0000_0000_0000_u128.to_le_bytes();

#[nrf_softdevice::gatt_service(uuid = "a8c1f8a0-0001-4b1a-8f7e-000000000000")]
pub struct LedService {
    #[characteristic(uuid = "a8c1f8a0-0001-4b1a-8f7e-000000000001", read, write, notify)]
    red_ctl: Payload,

    #[characteristic(uuid = "a8c1f8a0-0001-4b1a-8f7e-000000000002", read, write, notify)]
    green_ctl: Payload,

    #[characteristic(uuid = "a8c1f8a0-0001-4b1a-8f7e-000000000003", read, write, notify)]
    blue_ctl: Payload,
}

#[nrf_softdevice::gatt_server]
pub struct Server {
    led: LedService,
}

pub fn sd_config() -> nrf_softdevice::Config {
    nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            // The Xiao nRF52840 has no external 32.768 kHz crystal — use the internal RC.
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 1,
            event_length: 24,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 64 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: raw::BLE_GATTS_ATTR_TAB_SIZE_DEFAULT,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 1,
            central_role_count: 0,
            central_sec_count: 0,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: DEVICE_NAME.as_ptr() as _,
            current_len: DEVICE_NAME.len() as u16,
            max_len: DEVICE_NAME.len() as u16,
            write_perm: unsafe { mem::zeroed() },
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                raw::BLE_GATTS_VLOC_STACK as u8,
            ),
        }),
        ..Default::default()
    }
}

#[embassy_executor::task]
pub async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

pub async fn run(sd: &'static Softdevice, server: Server) -> ! {
    static ADV_DATA: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new()
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
        .services_128(ServiceList::Complete, &[LED_SERVICE_UUID_LE])
        .build();

    static SCAN_DATA: LegacyAdvertisementPayload =
        LegacyAdvertisementBuilder::new().full_name("XiaoLED").build();

    loop {
        let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
            adv_data: &ADV_DATA,
            scan_data: &SCAN_DATA,
        };
        let conn = match peripheral::advertise_connectable(
            sd,
            adv,
            &peripheral::Config::default(),
        )
        .await
        {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Make readback consistent: push the current authoritative state into
        // the characteristic value store so the client's first reads match.
        let _ = server.led.red_ctl_set(&state::RED.encode());
        let _ = server.led.green_ctl_set(&state::GREEN.encode());
        let _ = server.led.blue_ctl_set(&state::BLUE.encode());

        let _ = gatt_server::run(&conn, &server, |evt| match evt {
            ServerEvent::Led(e) => handle_led_event(e),
        })
        .await;
        // disconnected → loop and re-advertise
    }
}

fn handle_led_event(e: LedServiceEvent) {
    match e {
        LedServiceEvent::RedCtlWrite(v)   => apply(&state::RED,   &v),
        LedServiceEvent::GreenCtlWrite(v) => apply(&state::GREEN, &v),
        LedServiceEvent::BlueCtlWrite(v)  => apply(&state::BLUE,  &v),
        // CCCD writes (subscribe/unsubscribe) — nothing to do, notifications
        // are pushed unconditionally when state changes.
        LedServiceEvent::RedCtlCccdWrite   { .. } => {}
        LedServiceEvent::GreenCtlCccdWrite { .. } => {}
        LedServiceEvent::BlueCtlCccdWrite  { .. } => {}
    }
}

fn apply(led: &LedState, v: &Payload) {
    if let Some((enabled, period)) = LedState::decode(v) {
        led.set(enabled, period);
    }
}
