use crate::config::pomodoro::Hooks;
use crate::models::pomodoro::Mode;

pub fn run_cmds(conf: &Hooks, state: Mode) {
    let cmd = match state {
        Mode::Focus => conf.focus.clone(),
        Mode::LongBreak => conf.long.clone(),
        Mode::ShortBreak => conf.short.clone(),
    };

    std::thread::spawn(move || {
        let Some(parts) = shlex::split(&cmd) else {
            log::error!("failed to parse hook command: {}", cmd);
            return;
        };
        let mut parts = parts.into_iter();
        let Some(prog) = parts.next() else { return };

        let output = std::process::Command::new(prog)
            .args(parts)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .output();

        if let Ok(output) = output
            && !output.status.success()
        {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::error!("hook command failed: {stderr}");
        }
    });
}
