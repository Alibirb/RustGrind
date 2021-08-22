use crate::common::EndstopIdentifier;
use crate::config::RustGrindConfig;
use crate::messages::EndstopHitMsg;
use crate::messages::Message;

use gpio_cdev::*;
use nix::poll::*;

type PollEventFlags = nix::poll::PollFlags;

use std::collections::HashMap;
use std::os::unix::io::AsRawFd;
use std::sync::mpsc::Sender;
use std::thread;



pub struct EndstopStatusClient {
	endstop_hit: HashMap<EndstopIdentifier, bool>,
}

impl EndstopStatusClient {
	pub fn new() -> Self {
		EndstopStatusClient{
			endstop_hit: HashMap::new(),
		}
	}

	pub fn is_endstop_hit(&self, endstop_id: EndstopIdentifier) -> Option<&bool> {
		self.endstop_hit.get(&endstop_id)
	}

	pub fn process_message(&mut self, msg: EndstopHitMsg) {
		self.endstop_hit.insert(msg.endstop, msg.value);
	}
}



struct EndstopChecker {
	chip_name: String,
	lines: HashMap<EndstopIdentifier, u32>,
	line_to_endstop_id: HashMap<u32, EndstopIdentifier>,
	msg_senders: Vec<Sender<Message>>,
}

impl EndstopChecker {
	pub fn new(initial_config : &RustGrindConfig, msg_senders: Vec<Sender<Message>>) -> Self {
		let mut line_to_endstop_id = HashMap::new();
		for (endstop_id, line) in initial_config.endstop_config.iter() {
			line_to_endstop_id.insert(line.clone(), endstop_id.clone());
		}
		EndstopChecker{
			chip_name: initial_config.gpio_chip_name.clone(),
			lines: initial_config.endstop_config.clone(),
			line_to_endstop_id,
			msg_senders,
		}
	}

	pub fn run(&mut self) {
		// Based off of https://github.com/rust-embedded/gpio-cdev/blob/master/examples/monitor.rs

		println!("Open chip name {}", self.chip_name);

		let mut chip = Chip::new(self.chip_name.clone()).unwrap();
		// Get event handles for each line to monitor.
		let mut evt_handles: Vec<LineEventHandle> = self.lines
			.values()
			.into_iter()
			.map(|pin_num| {
				let line = chip.get_line(*pin_num).unwrap();
				line.events(
					LineRequestFlags::INPUT,
					EventRequestFlags::BOTH_EDGES,
					"monitor",
				)
				.unwrap()
			})
			.collect();

		// Create a vector of file descriptors for polling
		let mut pollfds: Vec<PollFd> = evt_handles
			.iter()
			.map(|h| {
				PollFd::new(
					h.as_raw_fd(),
					PollEventFlags::POLLIN | PollEventFlags::POLLPRI,
				)
			})
			.collect();

		// TODO: check initial status of endstops and send out messages

		loop {
			// Poll for an event on any of the lines
			if poll(&mut pollfds, -1).unwrap() != 0 {
				// received data
				for i in 0..pollfds.len() {
					if let Some(revts) = pollfds[i].revents() {
						let h = &mut evt_handles[i];
						if revts.contains(PollEventFlags::POLLIN) {
							let value = h.get_value().unwrap();
							// Retrieve and clear the event
							// We don't need the event object itself, but it won't clear until we take it.
							h.get_event().unwrap();
							let endstop = self.line_to_endstop_id.get(&h.line().offset()).unwrap();
							for sender in &self.msg_senders {
								sender.send(Message::EndstopHitMsgType(EndstopHitMsg{endstop: *endstop, value: (value != 0)}));
							}
							println!("Got event for GPIO {}, new value {}", h.line().offset(), value);
						}
					}
				}
			}
		}
	}
}


pub fn init(initial_config : RustGrindConfig, msg_senders: Vec<Sender<Message>>) {
	let builder = thread::Builder::new().name("EndstopChecker".to_string());
	builder.spawn(move || {
		let mut checker = EndstopChecker::new(&initial_config, msg_senders);
		checker.run();
	}).unwrap();
}
