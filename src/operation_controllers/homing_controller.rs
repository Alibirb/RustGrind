use super::OperationController;
use super::OperationControllerData;
use super::OperationParameters;

use crate::common::Axis;
use crate::common::AxisEnd;
use crate::messages::Message;
use crate::messages::MoveAxisRelMsg;
use crate::messages::MovementCompleteMsg;

use strum_macros::Display;



pub struct HomingParams {}
impl OperationParameters for HomingParams {
	fn make_controller(&self, data: OperationControllerData) -> Box<dyn OperationController> {
		Box::new(HomingController::new(data))
	}
}

#[derive(Copy, Clone)]
#[derive(Display)]
#[derive(PartialEq)]
enum HomingState {
	XMinus,
	XPlus,
	YMinus,
	YPlus,
}

struct HomingController {
	common_data: OperationControllerData,
	state: HomingState,
}
impl HomingController {
	pub fn new(common_data: OperationControllerData) -> Self {
		let mut ret = Self{
			common_data,
			state: HomingState::XMinus,
		};
		ret.set_state(HomingState::XMinus);
		ret
	}

	fn handle_movement_complete(&mut self, msg: MovementCompleteMsg) {
		if !msg.endstop_hit {
			// Didn't reach the endstop. Keep going.
			self.set_state(self.state);
			return;
		}
		match self.state {
			HomingState::XMinus => {
				self.work_envelope_mut().min_x = self.position_client().get_axis_position(Axis::X);
				self.set_state(HomingState::XPlus);
			},
			HomingState::XPlus => {
				self.work_envelope_mut().max_x = self.position_client().get_axis_position(Axis::X);
				self.set_state(HomingState::YMinus);
			},
			HomingState::YMinus => {
				self.work_envelope_mut().min_y = self.position_client().get_axis_position(Axis::Y);
				self.set_state(HomingState::YPlus);
			},
			HomingState::YPlus => {
				self.work_envelope_mut().max_y = self.position_client().get_axis_position(Axis::Y);
				self.stop();
			},
		}
	}

	fn set_state(&mut self, state: HomingState) {
		println!("Setting state to {}", state);
		self.state = state;
		match self.state {
			HomingState::XMinus => self.move_towards_extent(Axis::X, AxisEnd::Min),
			HomingState::XPlus => self.move_towards_extent(Axis::X, AxisEnd::Max),
			HomingState::YMinus => self.move_towards_extent(Axis::Y, AxisEnd::Min),
			HomingState::YPlus => self.move_towards_extent(Axis::Y, AxisEnd::Max),
		}
	}

	fn move_towards_extent(&mut self, axis: Axis, end: AxisEnd) {
		match end {
			AxisEnd::Max => self.send_to_motor_control(Message::MoveAxisRelMsgType(MoveAxisRelMsg{axis, distance: 256.0, speed: self.get_homing_speed(axis)})),
			AxisEnd::Min => self.send_to_motor_control(Message::MoveAxisRelMsgType(MoveAxisRelMsg{axis, distance: -256.0, speed: self.get_homing_speed(axis)})),
		}
	}

	fn get_homing_speed(&self, axis: Axis) -> f64 {
		self.config_client().config.motor_configs.get(&axis).unwrap().default_speed_ips
	}
}
impl OperationController for HomingController {
	fn operation_controller_data(&self) -> &OperationControllerData {
		&self.common_data
	}

	fn operation_controller_data_mut(&mut self) -> &mut OperationControllerData {
		&mut self.common_data
	}

	fn handle_message(&mut self, msg : Message) {
		match msg {
			Message::CurrentPositionMsgType(cp_msg) => self.position_client_mut().handle_message(cp_msg),
			Message::EndstopHitMsgType(eh_msg) => self.endstop_status_client_mut().process_message(eh_msg),

			Message::MovementCompleteMsgType(mc_msg) => self.handle_movement_complete(mc_msg),
			Message::StopMsgType() => self.stop(),

			_ => {}
		};
	}
}
