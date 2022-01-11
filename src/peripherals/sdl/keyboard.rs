use std::{collections::HashMap, time::Duration};

use sdl2::{keyboard::Keycode, event::Event, Sdl, EventPump};

use crate::arch::NUM_KEYS;

pub struct Keyboard {
    event_pump: EventPump,
    key_states: [bool;NUM_KEYS],
    reset: bool,
    power_off: bool,
    keymap: HashMap<Keycode, usize>,
    polling_interval: Duration
}

impl Keyboard {
    pub fn new(sdl: &Sdl) -> Option<Keyboard> {
        let event_pump = sdl.event_pump().ok()?;
        let mut keymap = HashMap::new();
        let keycodes = vec![
            Keycode::Q, Keycode::W, Keycode::E, Keycode::R,
            Keycode::A, Keycode::S, Keycode::D, Keycode::F,
            Keycode::Z, Keycode::X, Keycode::C, Keycode::V,
            Keycode::Num1, Keycode::Num2, Keycode::Num3, Keycode::Num4
        ];
        for (keycode, key) in keycodes.iter().zip(0x0..=0xF) {
            keymap.insert(*keycode, key);
        }
        let keyboard = Keyboard {
            event_pump,
            key_states: [false;NUM_KEYS],
            reset: false,
            power_off: false,
            keymap,
            polling_interval: Duration::from_millis(10)
        };
        Some(keyboard)
    }

    // Returns true if a key down event was found, otherwise false.
    fn process_events(&mut self) -> Option<u8> {
        let mut recently_pressed = None;
        self.event_pump.poll_iter().collect::<Vec<Event>>().iter().for_each(|evt| {
            match evt {
                sdl2::event::Event::Quit { .. } => {
                    self.power_off = true
                }
                sdl2::event::Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    self.reset = true
                }
                sdl2::event::Event::KeyDown { keycode: Some(keycode), .. } if self.keymap.contains_key(keycode) => {
                    let key = self.keymap[keycode];
                    self.key_states[key] = true;
                    recently_pressed = Some(key as u8);
                },
                sdl2::event::Event::KeyUp { keycode: Some(keycode), .. } if self.keymap.contains_key(keycode) => {
                    self.key_states[self.keymap[keycode]] = false
                }
                _ => { }
            }
        });
        recently_pressed
    }
}

impl crate::arch::Keyboard for Keyboard {
    fn pressed(&mut self, key: u8) -> bool {
        self.process_events();
        self.key_states[key as usize]
    }

    fn wait_key(&mut self) -> u8 {
        while !self.reset && !self.power_off {
            match self.process_events() {
                Some(key) => return key,
                None => std::thread::sleep(self.polling_interval)
            }
        }
        0
    }

    fn reset_signal(&mut self) -> bool {
        let reset = self.reset;
        self.reset = false;
        reset
    }

    fn power_off_signal(&mut self) -> bool {
        let power_off = self.power_off;
        self.power_off = false;
        power_off
    }

    fn reset(&mut self) {
        self.key_states = [false;NUM_KEYS];
    }
}
