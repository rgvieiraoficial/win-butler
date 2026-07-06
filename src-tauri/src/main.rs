// Em release, esconde o console (windows_subsystem). Em debug, mantém para logs.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    win_butler_lib::run();
}
