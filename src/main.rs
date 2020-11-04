use std::env::var;
use std::fs::remove_file;
use std::sync::{Arc, Mutex};

use clap::{crate_version, load_yaml, App};
use swayipc::reply::Event::Window;
use swayipc::reply::WindowChange;
use swayipc::{Connection, EventType};

type Res<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn get_current_focused_id() -> Res<i64> {
    Connection::new()?
        .get_tree()?
        .find_focused_as_ref(|n| n.focused)
        .map(|n| n.id)
        .ok_or_else(|| Err("Failed to get current Focused ID").unwrap())
}

fn handle_signal(last_focused: &Arc<Mutex<i64>>) -> Res<()> {
    Connection::new()?.run_command(format!("[con_id={}] focus", last_focused.lock().unwrap()))?;
    Ok(())
}

fn unbind_key() -> Res<()> {
    let yml = load_yaml!("args.yml");
    let args = App::from_yaml(yml).version(crate_version!()).get_matches();
    let key_combo = args.value_of("combo").unwrap_or("Mod1+Tab");
    
    let pid_file = format!(
        "{}/sway-alttab.pid",
        var("XDG_RUNTIME_DIR").unwrap_or("/tmp".to_string())
    );
    Connection::new()?.run_command(format!(
        "unbindsym {} exec pkill -USR1 -F {}",
        key_combo, pid_file
    ))?;
    Ok(())
}

fn bind_key() -> Res<()> {
    let yml = load_yaml!("args.yml");
    let args = App::from_yaml(yml).version(crate_version!()).get_matches();
    let key_combo = args.value_of("combo").unwrap_or("Mod1+Tab");

    let pid_file = format!(
        "{}/sway-alttab.pid",
        var("XDG_RUNTIME_DIR").unwrap_or("/tmp".to_string())
    );

    Connection::new()?.run_command(format!(
        "bindsym {} exec pkill -USR1 -F {}",
        key_combo, pid_file
    ))?;
    Ok(())
}

fn start_daemon() -> Res<()> {
    let dir = var("XDG_RUNTIME_DIR").unwrap_or("/tmp".to_string());

    unsafe { signal_hook::register(signal_hook::SIGTERM, cleanup)? };
    unsafe { signal_hook::register(signal_hook::SIGINT, cleanup)? };

    Ok(daemonize::Daemonize::new()
        .pid_file(format!("{}/sway-alttab.pid", dir))
        .chown_pid_file(true)
        .working_directory(dir)
        .start()?)
}

fn cleanup() {
    let dir = var("XDG_RUNTIME_DIR").unwrap_or("/tmp".to_string());
    remove_file(format!("{}/sway-alttab.pid", dir)).unwrap();
    unbind_key().unwrap();
    println!("Exiting sway-alttab");
}

fn main() -> Res<()> {
    let last_focus = Arc::new(Mutex::new(0));
    let mut cur_focus = get_current_focused_id()?;
    let clone = Arc::clone(&last_focus);

    unsafe {
        signal_hook::register(signal_hook::SIGUSR1, move || {
            handle_signal(&clone).unwrap();
        })?
    };

    start_daemon()?;

    bind_key()?;

    let subs = [EventType::Window];
    let mut events = Connection::new()?.subscribe(&subs)?;

    loop {
        let event = events.next();
        if let Some(Ok(Window(ev))) = event {
            if ev.change == WindowChange::Focus {
                let mut last = last_focus.lock().unwrap();
                *last = cur_focus;
                cur_focus = ev.container.id;
            }
        } else {
            cleanup();
        }
    }
}
