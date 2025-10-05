use chrono::NaiveDateTime;
use common_data::{
    request::{Request, MAX_REQUEST_SIZE},
    response::Response,
};
use link_lib::*;
use nb::block;
use rtic::Mutex;
use stm32l0xx_hal::{
    exti::{DirectLine, Exti},
    prelude::{_embedded_hal_serial_Read, _embedded_hal_serial_Write},
    serial::*,
};

use crate::{app::uart0, cfg::UART_SLEEP_DELAY};

pub struct LocalUartState {
    last_message_ts: i64,
    message_buffer: link_lib::Link<_, Request, Response, MAX_REQUEST_SIZE>,
}

pub fn uart_interrupt(mut ctx: uart0::Context) {
    if Exti::is_pending(DirectLine::Lpuart1) {
        //let state = ctx.local.uart_int_state;
        let now = ctx.shared.rtc.lock(|r| r.now().timestamp());
        let delta_time = now - ctx.local.uart_state.last_message_ts;
        ctx.local.uart_state.last_message_ts = now;

        // Clear buffer if we haven't received a byte for more than UART_SLEEP_DELAY secs, this is
        // needed because we need to clear the buffer if we receive a corrupted message
        // without a null byte at the end
        if delta_time > UART_SLEEP_DELAY {
            ctx.local.uart_state.message_buffer.clear_buffer();
        }

        while ctx.local.uart_int_state.uart_rx.is_rx_not_empty() {
            let byte = match block!(ctx.local.uart_int_state.uart_rx.read()) {
                Ok(b) => b,
                Err(_) => break,
            };
        }
    }
}

/*fn set_time(ctx: &mut uart0::Context, timestamp: u32) -> Response {
    let new_time = match NaiveDateTime::from_timestamp_opt(timestamp as i64, 0) {
        Some(t) => t,
        None => return Response::Error,
    };

    return match ctx.shared.rtc.lock(|rtc| rtc.set(new_time)) {
        Ok(_) => Response::Ok,
        Err(_) => Response::Error,
    };
}

fn get_sensor_id(ctx: &mut uart0::Context) -> Response {
    match ctx.shared.eeprom.lock(|eeprom| read_sensor_id(eeprom)) {
        Ok(id) => Response::SensorId(id),
        Err(_) => Response::Error,
    }
}

fn set_sensor_id(ctx: &mut uart0::Context, sensor_id: u64) -> Response {
    match ctx
        .shared
        .eeprom
        .lock(|eeprom| write_sensor_id(eeprom, sensor_id))
    {
        Ok(_) => Response::Ok,
        Err(_) => Response::Error,
    }
}

fn clear_memory(ctx: &mut uart0::Context) -> Response {
    match ctx.shared.eeprom.lock(|eeprom| zero_stored_count(eeprom)) {
        Ok(_) => Response::Ok,
        Err(_) => Response::Error,
    }
}

fn get_number_of_event(ctx: &mut uart0::Context) -> Response {
    match ctx
        .shared
        .eeprom
        .lock(|eeprom| read_count_from_eeprom(eeprom))
    {
        Ok(_) => Response::Ok,
        Err(_) => Response::Error,
    }
}

fn get_event(ctx: &mut uart0::Context, number: u32) -> Response {
    match ctx
        .shared
        .eeprom
        .lock(|eeprom| read_event_from_eeprom(eeprom, number))
    {
        Ok(e) => Response::EventInfo(e),
        Err(_) => Response::Error,
    }
}*/

pub struct UartInterruptState {
    pub uart_rx: Rx<LPUART1>,
    pub uart_tx: Tx<LPUART1>,
    pub last_timestamp: u32,
}
