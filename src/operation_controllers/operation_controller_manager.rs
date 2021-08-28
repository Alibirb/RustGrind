use crate::config::ConfigClient;
use crate::config::RustGrindConfig;
use crate::endstop_checker::EndstopStatusClient;
use crate::messages::Message;
use crate::motor_control::CurrentPositionClient;

use super::manual_control_controller::NoOpOperationParams;
use super::OperationController;
use super::OperationControllerData;
use super::OperationParameters;
use super::WorkEnvelope;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;



pub struct OperationControllerManager {
	controller: Box<dyn OperationController>,
	receiver: Receiver<Message>,
}

impl OperationControllerManager {
	pub fn new(config: RustGrindConfig, receiver: Receiver<Message>, motor_control_sender: Sender<Message>) -> Self {
		OperationControllerManager {
			controller: NoOpOperationParams{}.make_controller(
				OperationControllerData{
					config_client: ConfigClient::new(config),
					endstop_status_client: EndstopStatusClient::new(),
					position_client: CurrentPositionClient::new(),
					motor_control_sender,
					work_envelope: WorkEnvelope::new(),
					pending_operation_params: None,
				},
			),
			receiver,
		}
	}

	fn handle_message(&mut self, msg: Message) {
		self.controller.handle_message(msg);
		self.check_replace_controller();
	}

	fn check_replace_controller(&mut self) {
		let param_option_clone = &self.controller.operation_controller_data_mut().pending_operation_params.take();

		match param_option_clone {
			Some(params) => {
				self.controller = params.make_controller(self.controller.operation_controller_data().clone());
			},
			None => {}
		}
	}

	fn shutdown(&mut self) {
		println!("Shutting down operation control");
		self.controller.stop();
	}

	pub fn run(&mut self) -> ! {
		loop {
			// Check for messages
			loop {
				match self.receiver.try_recv() {
					Ok(msg) => self.handle_message(msg),
					Err(TryRecvError::Empty) => break,
					// If nothing is connected to send us messages, then something has gone very wrong.
					// There is no way for anything to get connected again, either.
					Err(TryRecvError::Disconnected) => self.shutdown(),
				}
			}

			self.controller.update();
			self.check_replace_controller();

			std::thread::sleep(Duration::from_nanos(5));
		}
	}
}
