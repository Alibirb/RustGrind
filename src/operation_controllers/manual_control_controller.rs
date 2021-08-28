use super::OperationController;
use super::OperationControllerData;
use super::OperationParameters;

use crate::messages::Message;



pub struct NoOpOperationParams {}
impl OperationParameters for NoOpOperationParams {
	fn make_controller(&self, data: OperationControllerData) -> Box<dyn OperationController> {
		Box::new(ManualControlController::new(data))
	}
}

struct ManualControlController {
	common_data: OperationControllerData
}
impl ManualControlController {
	pub fn new(common_data: OperationControllerData) -> Self {
		Self{
			common_data,
		}
	}
}
impl OperationController for ManualControlController {
	fn operation_controller_data(&self) -> &OperationControllerData {
		&self.common_data
	}

	fn operation_controller_data_mut(&mut self) -> &mut OperationControllerData {
		&mut self.common_data
	}
	
	fn stop(&mut self) {
		// Same as default implementation, except no need to replace controller because we're already the idle controller
		println!("Stopping all movement");
		self.send_to_motor_control(Message::StopMsgType());
	}

	fn handle_message(&mut self, msg : Message) {
		match msg {
			Message::CurrentPositionMsgType(cp_msg) => self.position_client_mut().handle_message(cp_msg),
			Message::EndstopHitMsgType(eh_msg) => self.endstop_status_client_mut().process_message(eh_msg),

			Message::GoToPositionMsgType(_) => self.send_to_motor_control(msg),
			Message::MoveAxisRelMsgType(_) => self.send_to_motor_control(msg),
			Message::SpindleControlMsgType(_) => self.send_to_motor_control(msg),
			Message::StopMsgType() => self.stop(),

			Message::StartSurfaceGrinderCutMsgType(cut_params) => self.change_controller(Box::new(cut_params)),

			_ => {}
		};
	}
}
