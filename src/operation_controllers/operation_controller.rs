use crate::config::ConfigClient;
use crate::endstop_checker::EndstopStatusClient;
use crate::messages::Message;
use crate::motor_control::CurrentPositionClient;

use super::manual_control_controller::NoOpOperationParams;
use super::OperationControllerData;
use super::WorkEnvelope;



pub trait OperationController {
	fn operation_controller_data(&self) -> &OperationControllerData;
	fn operation_controller_data_mut(&mut self) -> &mut OperationControllerData;

	fn config_client(&self) -> &ConfigClient {
		&self.operation_controller_data().config_client
	}

	fn endstop_status_client(&self) -> &EndstopStatusClient {
		&self.operation_controller_data().endstop_status_client
	}
	fn endstop_status_client_mut(&mut self) -> &mut EndstopStatusClient {
		&mut self.operation_controller_data_mut().endstop_status_client
	}

	fn position_client(&self) -> &CurrentPositionClient {
		&self.operation_controller_data().position_client
	}
	fn position_client_mut(&mut self) -> &mut CurrentPositionClient {
		&mut self.operation_controller_data_mut().position_client
	}

	fn work_envelope(&self) -> &WorkEnvelope {
		&self.operation_controller_data().work_envelope
	}

	fn send_to_motor_control(&self, msg: Message) {
		self.operation_controller_data().motor_control_sender.send(msg);
	}

	fn update(&mut self) {}

	fn stop(&mut self) {
		println!("Stopping all movement");
		self.send_to_motor_control(Message::StopMsgType());
		// Change to idle/manual control controller
		self.change_controller(Box::new(NoOpOperationParams{}));
	}

	fn handle_message(&mut self, msg: Message);

	fn change_controller(&mut self, params: Box<dyn OperationParameters>) {
		self.operation_controller_data_mut().pending_operation_params = Some(params);
	}
}

pub trait OperationParameters {
	fn make_controller(&self, data: OperationControllerData) -> Box<dyn OperationController>;
}
