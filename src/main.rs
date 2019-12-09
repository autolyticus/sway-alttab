use std::sync::{Arc, Mutex};

use swayipc::reply::Event::Window;
use swayipc::reply::WindowChange;
use swayipc::EventIterator;
use swayipc::{Connection, EventType};

type Res<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn get_current_focused_id() -> Res<i64> {
    Connection::new()?
        .get_tree()?
        .find_focused_as_ref(|n| n.focused)
        .map(|n| n.id)
        .ok_or_else(|| Err("Failed to get current Focused ID").unwrap())
}

fn handle_signal(last_focused: &Arc<Mutex<i64>>) {
    Connection::new()
        .unwrap()
        .run_command(format!(
            "[con_id={}] focus",
            (*last_focused).lock().unwrap()
        ))
        .unwrap();
}

fn get_events() -> Res<EventIterator> {
    let subs = [EventType::Window];
    let sub = Connection::new()?;
    Ok(sub.subscribe(&subs)?)
}

fn main() -> Res<()> {
    let last_focus = Arc::new(Mutex::new(0));
    let mut cur_focus = get_current_focused_id()?;
    let clone = Arc::clone(&last_focus);

    unsafe { signal_hook::register(signal_hook::SIGUSR1, move || handle_signal(&clone))? };

    let mut events = get_events()?;
    loop {
        let event = events.next();
        if let Some(Ok(Window(ev))) = event {
            if ev.change == WindowChange::Focus {
                let mut last = last_focus.lock().unwrap();
                *last = cur_focus;
                cur_focus = ev.container.id;
            }
        }
    }
}
