use rdev::{listen, EventType, Key};
use eframe::egui;
use tokio::sync::mpsc::UnboundedSender;



// Simple enum to describe key events sent from the listener to the UI
#[derive(Debug, Clone)]
pub enum KeyEvent {
    Toggle,
    Play,
    Pause,
    Next,
    Previous,
    Volup,
    Voldown,
    Mute,
}

pub fn capture_key_input(ctx: &egui::Context) -> Option<String> {
    
    for event in &ctx.input(|i| i.events.clone()) {
        if let egui::Event::Key { key, pressed: true, .. } = event {
            let modifiers = ctx.input(|i| i.modifiers);
            
            
            let mut combo = String::new();
            
            if modifiers.ctrl {
                combo.push_str("Ctrl+");
            }
            if modifiers.shift {
                combo.push_str("Shift+");
            }
            if modifiers.alt {
                combo.push_str("Alt+");
            }
            
            // for egui key events (capture), use debug name like "A" or "Enter"
            combo.push_str(&format!("{:?}", key));
            
            return Some(combo);
        }
    }
    None
}

pub fn listenforkey_send(
    tx: UnboundedSender<KeyEvent>,
    toggle_key_str: String,
    play_key_str: String,
    pause_key_str: String,
    next_key_str: String,
    previous_key_str: String,
    volup_key_str: String,
    voldown_key_str: String,
    mute_key_str: String,
) {
    
    
    // Parse the keybinds using str_to_key
    let toggle_key = str_to_key(&toggle_key_str);
    let play_key = str_to_key(&play_key_str);
    let pause_key = str_to_key(&pause_key_str);
    let next_key = str_to_key(&next_key_str);
    let previous_key = str_to_key(&previous_key_str);
    let volup_key = str_to_key(&volup_key_str);
    let voldown_key = str_to_key(&voldown_key_str);
    let mute_key = str_to_key(&mute_key_str);

    // Debug: print parsed binds
    eprintln!("[LISTENER] Parsed binds:");
    eprintln!("  Toggle: {} -> {:?}", toggle_key_str, toggle_key);
    eprintln!("  Play: {} -> {:?}", play_key_str, play_key);
    eprintln!("  Pause: {} -> {:?}", pause_key_str, pause_key);
    eprintln!("  Next: {} -> {:?}", next_key_str, next_key);
    eprintln!("  Previous: {} -> {:?}", previous_key_str, previous_key);
    eprintln!("  VolUp: {} -> {:?}", volup_key_str, volup_key);
    eprintln!("  VolDown: {} -> {:?}", voldown_key_str, voldown_key);
    eprintln!("  Mute: {} -> {:?}", mute_key_str, mute_key);

    
    // Create shared state for modifier keys
    let mut ctrl_pressed = false;
    let mut shift_pressed = false;
    let mut alt_pressed = false;
    
    if let Err(error) = listen(move |event| {
        match event.event_type {
            EventType::KeyPress(key) => {
                // Track modifier key states
                match key {
                    Key::ControlLeft | Key::ControlRight => {
                        ctrl_pressed = true;
                        return;
                    },
                    Key::ShiftLeft | Key::ShiftRight => {
                        shift_pressed = true;
                        return;
                    },
                    Key::Alt => {
                        alt_pressed = true;
                        return;
                    },
                    _ => {}
                }
                
                // Get current modifier states
                let has_ctrl = ctrl_pressed;
                let has_shift = shift_pressed;
                let has_alt = alt_pressed;
                
                println!("Detected key: {:?} | Ctrl: {}, Shift: {}, Alt: {}", key, has_ctrl, has_shift, has_alt);
                
                // Compare with keybinds (check key and modifiers match)

                if let Some(tk) = toggle_key {
                    if key == tk && has_ctrl == toggle_key_str.contains("Ctrl") && 
                       has_shift == toggle_key_str.contains("Shift") && has_alt == toggle_key_str.contains("Alt") {
                        let _ = tx.send(KeyEvent::Toggle);
                    }
                }
                if let Some(pk) = play_key {
                    if key == pk && has_ctrl == play_key_str.contains("Ctrl") && 
                       has_shift == play_key_str.contains("Shift") && has_alt == play_key_str.contains("Alt") {
                        let _ = tx.send(KeyEvent::Play);
                    }
                }
                if let Some(psk) = pause_key {
                    if key == psk && has_ctrl == pause_key_str.contains("Ctrl") && 
                       has_shift == pause_key_str.contains("Shift") && has_alt == pause_key_str.contains("Alt") {
                        let _ = tx.send(KeyEvent::Pause);
                    }
                }
                if let Some(nk) = next_key {
                    if key == nk && has_ctrl == next_key_str.contains("Ctrl") && 
                       has_shift == next_key_str.contains("Shift") && has_alt == next_key_str.contains("Alt") {
                        let _ = tx.send(KeyEvent::Next);
                    }
                }
                if let Some(pk) = previous_key {
                    if key == pk && has_ctrl == previous_key_str.contains("Ctrl") && 
                       has_shift == previous_key_str.contains("Shift") && has_alt == previous_key_str.contains("Alt") {
                        let _ = tx.send(KeyEvent::Previous);
                    }
                }
                if let Some(vu) = volup_key {
                    if key == vu && has_ctrl == volup_key_str.contains("Ctrl") && 
                       has_shift == volup_key_str.contains("Shift") && has_alt == volup_key_str.contains("Alt") {
                        let _ = tx.send(KeyEvent::Volup);
                    }
                }
                if let Some(vd) = voldown_key {
                    if key == vd && has_ctrl == voldown_key_str.contains("Ctrl") && 
                       has_shift == voldown_key_str.contains("Shift") && has_alt == voldown_key_str.contains("Alt") {
                        let _ = tx.send(KeyEvent::Voldown);
                    }
                }
                if let Some(mk) = mute_key {
                    if key == mk && has_ctrl == mute_key_str.contains("Ctrl") && 
                       has_shift == mute_key_str.contains("Shift") && has_alt == mute_key_str.contains("Alt") {
                        let _ = tx.send(KeyEvent::Mute);
                    }
                }
            },
            EventType::KeyRelease(key) => {
                
                match key {
                    Key::ControlLeft | Key::ControlRight => {
                        ctrl_pressed = false;
                    },
                    Key::ShiftLeft | Key::ShiftRight => {
                        shift_pressed = false;
                    },
                    Key::Alt => {
                        alt_pressed = false;
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }) {
        println!("Error: {:?}", error);
    }
}


pub fn str_to_key(s: &str) -> Option<Key> {
    // Take last part after '+' and normalize. Accept values like "A" or "KeyA" or " KeyA "
    let key_part = s.split('+').last().unwrap_or(s).trim();
    let key_part = if key_part.to_uppercase().starts_with("KEY") {
        // strip leading "Key" prefix
        key_part[3..].trim()
    } else {
        key_part
    };

    println!("Parsing key part: '{}'", key_part);

    match key_part.to_uppercase().as_str() {
        "A" => Some(Key::KeyA),
        "B" => Some(Key::KeyB),
        "C" => Some(Key::KeyC),
        "D" => Some(Key::KeyD),
        "E" => Some(Key::KeyE),
        "F" => Some(Key::KeyF),
        "G" => Some(Key::KeyG),
        "H" => Some(Key::KeyH),
        "I" => Some(Key::KeyI),
        "J" => Some(Key::KeyJ),
        "K" => Some(Key::KeyK),
        "L" => Some(Key::KeyL),
        "M" => Some(Key::KeyM),
        "N" => Some(Key::KeyN),
        "O" => Some(Key::KeyO),
        "P" => Some(Key::KeyP),
        "Q" => Some(Key::KeyQ),
        "R" => Some(Key::KeyR),
        "S" => Some(Key::KeyS),
        "T" => Some(Key::KeyT),
        "U" => Some(Key::KeyU),
        "V" => Some(Key::KeyV),
        "W" => Some(Key::KeyW),
        "X" => Some(Key::KeyX),
        "Y" => Some(Key::KeyY),
        "Z" => Some(Key::KeyZ),

        "ENTER" | "RETURN" => Some(Key::Return),
        "ESC" | "ESCAPE" => Some(Key::Escape),
        "SPACE" => Some(Key::Space),
        "TAB" => Some(Key::Tab),
        "SHIFT" => Some(Key::ShiftLeft),
        "CTRL" | "CONTROL" => Some(Key::ControlLeft),
        "ALT" => Some(Key::Alt),
        "BACKSPACE" => Some(Key::Backspace),
        "DELETE" => Some(Key::Delete),
        "LEFT" => Some(Key::LeftArrow),
        "RIGHT" => Some(Key::RightArrow),
        "UP" => Some(Key::UpArrow),
        "DOWN" => Some(Key::DownArrow),
        "HOME" => Some(Key::Home),
        "END" => Some(Key::End),
        "PAGEUP" | "PageUpDetected" => Some(Key::PageUp),       // only god knows why 
        "PAGEDOWN" | "PageDownDetected "=> Some(Key::PageDown), // egui uses "detected" for pgup/down
        "PAUSE" => Some(Key::Pause),
        "INS" | "INSERT" => Some(Key::Insert),
        "DEL" => Some(Key::Delete),
        "FN" => Some(Key::Function),
        "SCROLLLOCk" => Some(Key::ScrollLock),
        "PRINTSCREEN" => Some(Key::PrintScreen),
        "EQUALS" => Some(Key::Equal),
        "MINUS" => Some(Key::Minus),
        "COMMA" => Some(Key::Comma),
        "SLASH" => Some(Key::Slash),
        "LEFTBRACKET" => Some(Key::LeftBracket),
        "RIGHTBRACKET" => Some(Key::RightBracket),
        "LEFTARROW" => Some(Key::LeftArrow),
        "RIGHTARROW" => Some(Key::RightArrow),
        "UPARROW" => Some(Key::UpArrow),
        "DOWNARROW" => Some(Key::DownArrow),


        "F1" => Some(Key::F1),
        "F2" => Some(Key::F2),
        "F3" => Some(Key::F3),
        "F4" => Some(Key::F4),
        "F5" => Some(Key::F5),
        "F6" => Some(Key::F6),
        "F7" => Some(Key::F7),
        "F8" => Some(Key::F8),
        "F9" => Some(Key::F9),
        "F10" => Some(Key::F10),
        "F11" => Some(Key::F11),
        "F12" => Some(Key::F12),

        _ => None,
    }
}

/* 
pub fn key_to_str(key: &Key) -> &'static str {
    match *key {
        Key::KeyA => "A",
        Key::KeyB => "B",
        Key::KeyC => "C",
        Key::KeyD => "D",
        Key::KeyE => "E",
        Key::KeyF => "F",
        Key::KeyG => "G",
        Key::KeyH => "H",
        Key::KeyI => "I",
        Key::KeyJ => "J",
        Key::KeyK => "K",
        Key::KeyL => "L",
        Key::KeyM => "M",
        Key::KeyN => "N",
        Key::KeyO => "O",
        Key::KeyP => "P",
        Key::KeyQ => "Q",
        Key::KeyR => "R",
        Key::KeyS => "S",
        Key::KeyT => "T",
        Key::KeyU => "U",
        Key::KeyV => "V",
        Key::KeyW => "W",
        Key::KeyX => "X",
        Key::KeyY => "Y",
        Key::KeyZ => "Z",

        Key::Return => "Enter",
        Key::Escape => "Escape",
        Key::Space => "Space",
        Key::Tab => "Tab",
        Key::ShiftLeft | Key::ShiftRight => "Shift",
        Key::ControlLeft | Key::ControlRight => "Ctrl",
        Key::Alt => "Alt",
        Key::Backspace => "Backspace",
        Key::Delete => "Delete",
        Key::LeftArrow => "Left",
        Key::RightArrow => "Right",
        Key::UpArrow => "Up",
        Key::DownArrow => "Down",

        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",

        _ => "Unknown",
    }
}
    */
