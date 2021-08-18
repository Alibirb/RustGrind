#![feature(proc_macro_hygiene, decl_macro)]
#![feature(option_result_contains)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;


mod common;
mod config;
mod endstop_checker;
mod messages;
mod motor_control;
mod pins;
mod surface_grinder_cut_controller;
mod ui;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;



fn main() -> ! {
	let (main_thread_sender, main_thread_receiver) = mpsc::channel();
	let (motor_control_sender, motor_control_receiver) = mpsc::channel();

	let mut config_manager = config::ConfigManager::new();
	// FIXME: log error if it fails to read the file
	config_manager.read_config_file();
	// REMOVEME: just writing this temporarily, so we have an up-to-date config file during initial development (format will change often at the moment)
	config_manager.write_config_file();
	let initial_config = config_manager.get_config();

	ui::init(main_thread_sender.clone());
	motor_control::init(initial_config.clone(), motor_control_receiver, main_thread_sender.clone());
	endstop_checker::init(initial_config.clone(), vec![motor_control_sender.clone(), main_thread_sender.clone()]);
	surface_grinder_cut_controller::init(initial_config.clone(), main_thread_receiver, motor_control_sender.clone());

	loop {
		thread::sleep(Duration::from_millis(1000));
	}
}
