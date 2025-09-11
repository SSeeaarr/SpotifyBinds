use global_hotkey::GlobalHotKeyEvent;

pub async fn listenforkey(){
    if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
    println!("{:?}", event);
}
}