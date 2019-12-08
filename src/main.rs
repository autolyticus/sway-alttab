use std::sync::{Arc, Mutex};

use swayipc::reply::Event::Window;
use swayipc::reply::WindowChange;
use swayipc::{block_on, Connection, EventType};

type Res<T> = std::result::Result<T, Box<dyn std::error::Error>>;

async fn get_current_focused_id() -> i64 {
    Connection::new()
        .await
        .unwrap()
        .get_tree()
        .await
        .unwrap()
        .find_focused_as_ref(|n| n.focused)
        .unwrap()
        .id
}

fn handle_signal(last_focused: &Arc<Mutex<i64>>) {
    block_on(async {
        Connection::new()
            .await
            .unwrap()
            .run_command(format!(
                "[con_id={}] focus",
                (*last_focused).lock().unwrap()
            ))
            .await
            .unwrap();
    })
}

fn main() -> Res<()> {
    block_on(async {
        let subs = [EventType::Window];
        let sub = Connection::new().await?;
        let mut events = sub.subscribe(&subs).await?;

        let last_focus = Arc::new(Mutex::new(0));
        let mut cur_focus = get_current_focused_id().await;
        let clone = Arc::clone(&last_focus);

        unsafe { signal_hook::register(signal_hook::SIGUSR1, move || handle_signal(&clone))? };
        loop {
            let event = events.next().await?;
            if let Window(ev) = event {
                if ev.change == WindowChange::Focus {
                    let mut last = last_focus.lock().unwrap();
                    *last = cur_focus;
                    cur_focus = ev.container.id;
                }
            }
        }
    })
}
