use color_eyre::eyre::Error;
use rppal::gpio::{Gpio, Trigger};
use std::time::Duration;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::mpsc::{self, Receiver};
use std::thread;

const APPEND: u8 = 20;
const NEXT_PIN: u8 = 16;
const PREV_PIN: u8 = 21;
const VOL_UP_PIN: u8 = 6;
const VOL_DOWN_PIN: u8 = 19;
const PLAY_PAUSE_PIN: u8 = 13;

const LEFT_PIN: u8 = 26;
const RIGHT_PIN: u8 = 5;

pub(crate) fn setup_gpio() -> Result<Receiver<KeyEvent>, Error> {
    let gpio = Gpio::new()?;
    let (tx, rx) = mpsc::channel();

    let mut play_pause_pin = gpio.get(PLAY_PAUSE_PIN)?.into_input_pullup();
    let mut append_pin = gpio.get(APPEND)?.into_input_pullup();
    let mut next_pin = gpio.get(NEXT_PIN)?.into_input_pullup();
    let mut prev_pin = gpio.get(PREV_PIN)?.into_input_pullup();
    let mut vol_up_pin = gpio.get(VOL_UP_PIN)?.into_input_pullup();
    let mut vol_down_pin = gpio.get(VOL_DOWN_PIN)?.into_input_pullup();
    let mut left_pin = gpio.get(LEFT_PIN)?.into_input_pullup();
    let mut right_pin = gpio.get(RIGHT_PIN)?.into_input_pullup();

    let tx1 = tx.clone();
    play_pause_pin.set_async_interrupt(
        Trigger::FallingEdge,
        Some(Duration::from_millis(33)),
        move |_| {
            let _ = tx1.send(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        },
    )?;

    let tx2 = tx.clone();
    next_pin.set_async_interrupt(
        Trigger::FallingEdge,
        Some(Duration::from_millis(33)),
        move |_| {
            let _ = tx2.send(KeyEvent::new(KeyCode::Char('K'), KeyModifiers::NONE));
        },
    )?;

    let tx3 = tx.clone();
    prev_pin.set_async_interrupt(
        Trigger::FallingEdge,
        Some(Duration::from_millis(33)),
        move |_| {
            let _ = tx3.send(KeyEvent::new(KeyCode::Char('J'), KeyModifiers::NONE));
        },
    )?;

    let tx4 = tx.clone();
    vol_up_pin.set_async_interrupt(
        Trigger::FallingEdge,
        Some(Duration::from_millis(33)),
        move |_| {
            let _ = tx4.send(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        },
    )?;

    let tx5 = tx.clone();
    vol_down_pin.set_async_interrupt(
        Trigger::FallingEdge,
        Some(Duration::from_millis(33)),
        move |_| {
            let _ = tx5.send(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
        },
    )?;

    let tx6 = tx.clone();
    append_pin.set_async_interrupt(
        Trigger::FallingEdge,
        Some(Duration::from_millis(33)),
        move |_| {
            let _ = tx6.send(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE));
        },
    )?;

    let tx7 = tx.clone();
    left_pin.set_async_interrupt(
        Trigger::FallingEdge,
        Some(Duration::from_millis(33)),
        move |_| {
            let _ = tx7.send(KeyEvent::new(KeyCode::Char('<'), KeyModifiers::NONE));
        },
    )?;

    let tx8 = tx.clone();
    right_pin.set_async_interrupt(
        Trigger::FallingEdge,
        Some(Duration::from_millis(33)),
        move |_| {
            let _ = tx8.send(KeyEvent::new(KeyCode::Char('>'), KeyModifiers::NONE));
        },
    )?;

    thread::spawn(move || {
        let _pins = (
            play_pause_pin,
            next_pin,
            prev_pin,
            vol_up_pin,
            vol_down_pin,
            append_pin,
            left_pin,
            right_pin,
        );
        loop {
            thread::sleep(Duration::from_millis(33));
        }
    });

    Ok(rx)
}
