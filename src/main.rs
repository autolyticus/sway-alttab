use std::env::var;
use std::fs::remove_file;
use std::sync::{Arc, Mutex};

use clap::{crate_version, load_yaml, App};
use i3ipc::{event::Event, reply::Node, I3Connection, I3EventListener, Subscription};

type Res<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn get_current_focused_id(node: &Node) -> Res<i64> {
    if node.focused == true {
        return Ok(node.id);
    }
    let Node {
        nodes,
        floating_nodes,
        focus,
        ..
    } = node;
    let first = *focus.first().ok_or("No focus")?;
    for node in nodes {
        if node.id == first {
            return get_current_focused_id(node);
        }
    }
    for node in floating_nodes {
        if node.id == first {
            return get_current_focused_id(node);
        }
    }
    Err("Failed to get currently focused id")?
}

fn handle_signal(last_focused: &Arc<Mutex<i64>>) -> Res<()> {
    println!("Received signal");
    I3Connection::connect()?.run_command(&format!("[con_id={}] focus", last_focused.lock().unwrap()))?;
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
    I3Connection::connect()?.run_command(&format!(
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

    I3Connection::connect()?.run_command(&format!(
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
    let mut cur_focus = get_current_focused_id(&I3Connection::connect()?.get_tree()?)?;
    let clone = Arc::clone(&last_focus);

    unsafe {
        signal_hook::register(signal_hook::SIGUSR1, move || {
            handle_signal(&clone).unwrap();
        })?
    };

    start_daemon()?;

    bind_key()?;

    let subs = [Subscription::Window];
    let mut listener = I3EventListener::connect()?;
    listener.subscribe(&subs)?;

    for event in listener.listen() {
        match event.unwrap() {
            Event::WindowEvent(e) => {
                println!("{:?}", e.container.id);
                let mut last = last_focus.lock().unwrap();
                *last = cur_focus;
                cur_focus = e.container.id;
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}
