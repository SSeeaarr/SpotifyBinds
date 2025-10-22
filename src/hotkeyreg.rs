use global_hotkey::{GlobalHotKeyManager, GlobalHotKeyEvent, hotkey::{Code, Modifiers, HotKey}};


pub async fn listenforkey(){
    let manager = GlobalHotKeyManager::new().unwrap();
    let hotkey = HotKey::new(Some(Modifiers::CONTROL), Code::KeyA);
    manager.register(hotkey).unwrap();
}