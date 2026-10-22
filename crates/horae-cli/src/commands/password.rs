use anyhow::{bail, Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use rusqlite::Connection;
use std::io::{stdout, IsTerminal, Write};

use crate::cli::PasswordAction;

/// 在终端中不回显读取密码输入。
pub fn prompt_password(prompt: &str) -> Result<String> {
    if !std::io::stdin().is_terminal() {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        return Ok(line.trim_end_matches(['\r', '\n']).to_string());
    }

    print!("{}", prompt);
    stdout().flush().context("flush stdout")?;

    enable_raw_mode().context("enable raw mode")?;
    let mut input = String::new();
    let res = (|| -> Result<()> {
        loop {
            if let Event::Key(key) = event::read().context("read event")? {
                match key.code {
                    KeyCode::Enter => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        bail!("cancelled by user");
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        input.clear();
                    }
                    KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        input.push(c);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    })();

    let _ = disable_raw_mode();
    println!();
    res?;

    Ok(input)
}

pub fn run(conn: &Connection, action: Option<PasswordAction>) -> Result<()> {
    let action = action.unwrap_or(PasswordAction::Status);
    match action {
        PasswordAction::Status => {
            let enabled = horae_core::password::is_password_enabled(conn)?;
            if enabled {
                println!("Password protection mode: ENABLED");
            } else {
                println!("Password protection mode: DISABLED (default)");
            }
            Ok(())
        }
        PasswordAction::Enable { password } => {
            let pass = match password {
                Some(p) => p,
                None => {
                    let p1 = prompt_password("Enter new password: ")?;
                    if p1.trim().is_empty() {
                        bail!("Password cannot be empty.");
                    }
                    let p2 = prompt_password("Confirm new password: ")?;
                    if p1 != p2 {
                        bail!("Passwords do not match.");
                    }
                    p1
                }
            };
            horae_core::password::set_password(conn, &pass)?;
            println!("✓ Password protection mode enabled.");
            Ok(())
        }
        PasswordAction::Disable { password } => {
            if !horae_core::password::is_password_enabled(conn)? {
                println!("Password protection mode is already disabled.");
                return Ok(());
            }
            let pass = match password {
                Some(p) => p,
                None => prompt_password("Enter current password: ")?,
            };
            if horae_core::password::verify_password(conn, &pass)? {
                horae_core::password::disable_password(conn)?;
                println!("✓ Password protection mode disabled.");
                Ok(())
            } else {
                bail!("Incorrect password.");
            }
        }
        PasswordAction::Change {
            old_password,
            new_password,
        } => {
            let old_p = match old_password {
                Some(p) => p,
                None => prompt_password("Enter current password: ")?,
            };
            if !horae_core::password::verify_password(conn, &old_p)? {
                bail!("Incorrect original password.");
            }
            let new_p = match new_password {
                Some(p) => p,
                None => {
                    let p1 = prompt_password("Enter new password: ")?;
                    if p1.trim().is_empty() {
                        bail!("Password cannot be empty.");
                    }
                    let p2 = prompt_password("Confirm new password: ")?;
                    if p1 != p2 {
                        bail!("Passwords do not match.");
                    }
                    p1
                }
            };
            horae_core::password::set_password(conn, &new_p)?;
            println!("✓ Password successfully updated.");
            Ok(())
        }
    }
}
