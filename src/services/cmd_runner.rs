use crate::config::PomodoroHookConfig;
use crate::models::pomodoro::PomodoroState;

pub fn run_cmds(conf: &PomodoroHookConfig, state: PomodoroState) {
    let cmd = match state {
        PomodoroState::Focus => conf.focus.clone(),
        PomodoroState::LongBreak => conf.long.clone(),
        PomodoroState::ShortBreak => conf.short.clone(),
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
