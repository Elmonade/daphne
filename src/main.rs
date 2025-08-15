use crate::button_handler::handle_config;
use crate::button_handler::handle_playback;
use crate::button_handler::handle_search;
use crate::state::Configure;
use crate::state::PlayerState;
use crate::utility::play_new_track;
use crate::view::render;
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use playback::SinkState;
use ratatui::DefaultTerminal;
use rppal::gpio::{Gpio, Trigger};
use serde::Deserialize;
use std::path::PathBuf;
use std::result::Result::Ok;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;
mod button_handler;
mod fuzzy_search;
mod order;
mod playback;
mod state;
mod utility;
mod view;

const PLAY_PAUSE_PIN: u8 = 20;
const NEXT_PIN: u8 = 16;
const PREV_PIN: u8 = 21;
const VOL_UP_PIN: u8 = 6;
const VOL_DOWN_PIN: u8 = 19;

#[derive(Deserialize)]
struct Config {
    path: PathBuf,
    seek_distance: usize,
}

#[derive(Debug, Clone)]
struct Audio {
    is_playing: bool,
    name: String,
    author: String,
    length: u64,
    path: PathBuf,
}

#[derive(Debug)]
pub(crate) enum Command {
    PlayPause(PathBuf),
    New(PathBuf),
    Forward(usize, usize),
    Backward(usize),
    Volume(f32),
    _Next(PathBuf, i32),
    _Previous(PathBuf, i32),
    _Append(PathBuf, i32),
}

enum Action {
    None,
    Submit,
    Escape,
}

fn setup_gpio_buttons() -> Result<Receiver<KeyEvent>> {
    let gpio = Gpio::new()?;
    let (tx, rx) = mpsc::channel();

    // Setup pins with pull-up resistors
    let mut play_pause_pin = gpio.get(PLAY_PAUSE_PIN)?.into_input_pullup();
    let mut next_pin = gpio.get(NEXT_PIN)?.into_input_pullup();
    let mut prev_pin = gpio.get(PREV_PIN)?.into_input_pullup();
    let mut vol_up_pin = gpio.get(VOL_UP_PIN)?.into_input_pullup();
    let mut vol_down_pin = gpio.get(VOL_DOWN_PIN)?.into_input_pullup();

    // Setup interrupts with hardware debouncing
    let tx1 = tx.clone();
    play_pause_pin.set_async_interrupt(Trigger::FallingEdge, Some(Duration::from_millis(100)), move |_| {
        let _ = tx1.send(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE));
    })?;

    let tx2 = tx.clone();
    next_pin.set_async_interrupt(Trigger::FallingEdge, Some(Duration::from_millis(100)), move |_| {
        let _ = tx2.send(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
    })?;

    let tx3 = tx.clone();
    prev_pin.set_async_interrupt(Trigger::FallingEdge, Some(Duration::from_millis(100)), move |_| {
        let _ = tx3.send(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
    })?;

    let tx4 = tx.clone();
    vol_up_pin.set_async_interrupt(Trigger::FallingEdge, Some(Duration::from_millis(100)), move |_| {
        let _ = tx4.send(KeyEvent::new(KeyCode::Char('K'), KeyModifiers::NONE));
    })?;

    let tx5 = tx.clone();
    vol_down_pin.set_async_interrupt(Trigger::FallingEdge, Some(Duration::from_millis(100)), move |_| {
        let _ = tx5.send(KeyEvent::new(KeyCode::Char('J'), KeyModifiers::NONE));
    })?;

    // Keep pins alive by moving them to a background thread
    thread::spawn(move || {
        let _pins = (play_pause_pin, next_pin, prev_pin, vol_up_pin, vol_down_pin);
        loop {
            thread::sleep(Duration::from_millis(100));
        }
    });

    Ok(rx)
}

fn main() -> Result<()> {
    env_logger::init();

    let mut state = if let Some(path) = home::home_dir() {
        let config_path = path.join(".config").join("daph.toml");
        if config_path.exists() {
            PlayerState::configured(config_path)
        } else {
            PlayerState::default()
        }
    } else {
        PlayerState::default()
    };

    state.table_state.select_first();
    state.table_state.select_first_column();
    state.list_state.select_first();

    let (command_tx, sink_rx) = playback::setup();
    state.tx = command_tx;
    state.sink_rx = sink_rx;

    let gpio_rx = setup_gpio_buttons()?;

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal, &mut state, gpio_rx);

    let _ = ratatui::try_restore();
    result
}

fn run(mut terminal: DefaultTerminal, state: &mut PlayerState, gpio_rx: Receiver<KeyEvent>) -> Result<()> {
    loop {
        if let Ok(sink) = state.sink_rx.recv_timeout(Duration::from_millis(33)) {
            // Render
            terminal.draw(|f| render(f, state, &sink))?;

            // Handle GPIO button input
            if let Ok(key) = gpio_rx.try_recv() {
                if state.is_searching {
                    match handle_search(key, state) {
                        Action::Escape => state.is_searching = false,
                        Action::Submit => {}
                        Action::None => {}
                    }
                } else if state.is_configuring {
                    match handle_config(key, state) {
                        Action::Escape => state.is_configuring = false,
                        Action::Submit => {}
                        Action::None => {}
                    }
                } else {
                    match handle_playback(key, state) {
                        Action::Escape => break,
                        Action::Submit => {}
                        Action::None => {}
                    }
                }
            }

            // Auto-Queue
            if sink.current_track_finished {
                if let Some(mut index) = state.current_track_index {
                    state.tracks[index].is_playing = false;
                    index = (index + 1) % state.number_of_tracks;
                    play_new_track(index, state);
                }
            }

            // If we assume two threads are perfectly in sync(probably impossible),
            // in total, one iteration should take 49ms when no button is pressed.
            // 2s / 49ms = ~41
            state.iteration_count += 1;
            if state.iteration_count % 41 == 0 {
                state.is_adjusting = false;
                state.iteration_count = 0;
            }
        }
    }
    Ok(())
}
