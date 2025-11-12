use rdev::{listen, Event, EventType, Key};
use tokio::sync::mpsc;
use std::sync::mpsc::channel;
use std::thread;

pub async fn listenforkey(spotify: AuthCodeSpotify) {
    todo!()

    
}

fn record_key() -> String {

    let (tx, rx) = channel();

    thread::spawn(move || {
        let _ = listen(move |event: Event| {
            if let EventType::KeyPress(key) = event.event_type {
                let _ = tx.send(key);
            }
        });
    });

    let key = rx.recv().unwrap(); // waits for first key
    format!("{:?}", key)
}

async fn register_ordered_key_combinations() -> Vec<String> {
    use rdev::{listen, Event, EventType};
    use tokio::sync::mpsc;
    use std::thread;

    let (tx, mut rx) = mpsc::channel(1);

    // Spawn blocking rdev listener
    thread::spawn(move || {
        let _ = listen(move |event: Event| {
            if let EventType::KeyPress(key) = event.event_type {
                let _ = tx.blocking_send(format!("{:?}", key));
            }
        });
    });

    // Wait for the first key press
    if let Some(key) = rx.recv().await {
        vec![key]
    } else {
        vec![]
    }
}

pub fn str_to_key(s: &str) -> Option<Key> {
    match s.to_uppercase().as_str() {
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


pub fn key_to_str(key: Key) -> &'static str {
    match key {
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
