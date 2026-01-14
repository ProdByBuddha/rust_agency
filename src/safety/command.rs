//! Command Safety Heuristics
//! 
//! Based on codex-rs patterns for identifying safe and dangerous shell commands.

pub fn is_known_safe_command(command: &[String]) -> bool {
    let Some(cmd0) = command.first().map(|s| s.as_str()) else {
        return false;
    };

    // Normalize zsh to bash for consistency in checks
    let cmd0_normalized = if cmd0 == "zsh" { "bash" } else { cmd0 };

    match std::path::Path::new(cmd0_normalized)
        .file_name()
        .and_then(|osstr| osstr.to_str())
    {
        Some(
            "cat" | "cd" | "cut" | "echo" | "expr" | "false" | "grep" | "head" |
            "id" | "ls" | "nl" | "paste" | "pwd" | "rev" | "seq" | "stat" |
            "tail" | "tr" | "true" | "uname" | "uniq" | "wc" | "which" | "whoami"
        ) => true,

        Some("git") => matches!(
            command.get(1).map(|s| s.as_str()),
            Some("branch" | "status" | "log" | "diff" | "show")
        ),

        Some("cargo") => matches!(
            command.get(1).map(|s| s.as_str()),
            Some("check" | "test" | "run" | "build")
        ),

        Some("find") => {
            // Unsafe find options that can delete or execute
            const UNSAFE_FIND: &[&str] = &["-exec", "-execdir", "-ok", "-okdir", "-delete", "-fls", "-fprint"];
            !command.iter().any(|arg| UNSAFE_FIND.contains(&arg.as_str()))
        }

        Some("rg") => {
            // Unsafe ripgrep options
            const UNSAFE_RG: &[&str] = &["--pre", "--hostname-bin", "--search-zip", "-z"];
            !command.iter().any(|arg| UNSAFE_RG.contains(&arg.as_str()))
        }

        _ => false,
    }
}

pub fn is_dangerous_command(command: &[String]) -> bool {
    if is_known_safe_command(command) {
        return false;
    }

    if is_dangerous_to_call_with_exec(command) {
        return true;
    }

    // Check for sudo
    if let Some(cmd0) = command.first() {
        if cmd0 == "sudo" {
            return is_dangerous_command(&command[1..]);
        }
    }

    // Check for shell wrappers like bash -c or bash -lc
    if let Some(cmd0) = command.first() {
        if matches!(cmd0.as_str(), "bash" | "sh" | "zsh") {
            for (i, arg) in command.iter().enumerate() {
                if (arg == "-c" || arg == "-lc") && i + 1 < command.len() {
                    let script = &command[i + 1];
                    if script_contains_dangerous_pattern(script) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

fn is_dangerous_to_call_with_exec(command: &[String]) -> bool {
    let cmd0 = command.first().map(|s| s.as_str());

    match cmd0 {
        Some(cmd) if cmd.ends_with("git") || cmd.ends_with("/git") => {
            matches!(command.get(1).map(|s| s.as_str()), Some("reset" | "rm" | "push"))
        }

        Some("rm") => {
            command.iter().any(|arg| arg == "-rf" || arg == "-f" || arg == "-r")
        }

        Some("mv") => {
            command.iter().any(|arg| arg.starts_with("/etc") || arg.starts_with("/bin") || arg.starts_with("/usr"))
        }

        Some("dd" | "mkfs" | "fdisk" | "reboot" | "shutdown") => true,
        
        Some("kill") => {
            command.iter().any(|arg| arg == "-9" || arg == "1")
        }

        _ => false,
    }
}

fn script_contains_dangerous_pattern(script: &str) -> bool {
    let dangerous_patterns = [
        "rm -rf", "rm -f", "rm -r",
        "git reset", "git rm", "git push",
        ":(){ :|:& };:", // Fork bomb
        "> /dev/sda",
        "mkfs",
        "dd if=",
        "chmod -R 777",
        "chown -R",
    ];

    for pattern in dangerous_patterns {
        if script.contains(pattern) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_commands() {
        assert!(is_known_safe_command(&vec!["ls".to_string(), "-la".to_string()]));
        assert!(is_known_safe_command(&vec!["git".to_string(), "status".to_string()]));
        assert!(!is_known_safe_command(&vec!["rm".to_string(), "-rf".to_string(), "/".to_string()]));
    }

    #[test]
    fn test_dangerous_commands() {
        assert!(is_dangerous_command(&vec!["rm".to_string(), "-rf".to_string(), "/".to_string()]));
        assert!(is_dangerous_command(&vec!["sudo".to_string(), "git".to_string(), "reset".to_string(), "--hard".to_string()]));
        assert!(is_dangerous_command(&vec!["bash".to_string(), "-c".to_string(), "rm -rf .".to_string()]));
        assert!(!is_dangerous_command(&vec!["ls".to_string(), "-la".to_string()]));
    }
}
