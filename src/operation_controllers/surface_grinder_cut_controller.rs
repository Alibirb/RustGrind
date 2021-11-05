use crate::common::Axis;
use crate::common::AxisEnd;
use crate::common::EndstopIdentifier;
use crate::messages::GoToPositionMsg;
use crate::messages::Message;
use crate::messages::MoveAxisRelMsg;
use crate::messages::MovementCompleteMsg;
use crate::messages::SpindleControlMsg;

use super::OperationController;
use super::OperationControllerData;
use super::OperationParameters;

use std::time::Duration;
use std::time::Instant;

use strum_macros::Display;



#[derive(Copy, Clone)]
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct SurfaceGrinderCutParams {
	pub depth_of_cut: f64,	// Z depth of each pass
	pub feed_per_pass: f64,	// Y feed per pass
	pub stroke_speed: f64,	// IPS
	pub total_depth: f64,
}

impl SurfaceGrinderCutParams {
	pub fn new() -> Self {
		SurfaceGrinderCutParams {
			depth_of_cut: 0.0,
			feed_per_pass: 0.0,
			stroke_speed: 0.0,
			total_depth: 0.0,
		}
	}
}

impl OperationParameters for SurfaceGrinderCutParams {
	fn make_controller(&self, data: OperationControllerData) -> Box<dyn OperationController> {
		Box::new(SurfaceGrinderCutController::new(data, *self))
	}
}



#[derive(Display)]
#[derive(PartialEq)]
enum SurfaceGrinderCutState {
	Idle,
	ToStartingPositionX,
	ToStartingPositionY,
	SpindleSpinUp,
	XCut,
	XReturn,
	YReturn,
	YOut,
	ZDown,
}

use SurfaceGrinderCutState as CutState;


/**
 * Cut controller for a surface grinder.
 */
struct SurfaceGrinderCutController {
	common_data: OperationControllerData,
	cut_params: SurfaceGrinderCutParams,
	state: CutState,
	spindle_started_time: Instant,
	starting_height: f64,
}

impl OperationController for SurfaceGrinderCutController {
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

			_ => {},
		};
	}

	fn update(&mut self) {
		if
			self.state == CutState::SpindleSpinUp
			&& self.spindle_started_time.elapsed() >= Duration::from_secs(3)
		{
			self.advance_state();
		}
	}
}

impl SurfaceGrinderCutController {
	pub fn new(common_data: OperationControllerData, cut_params: SurfaceGrinderCutParams) -> Self {
		let mut ret = SurfaceGrinderCutController {
			common_data,
			cut_params,
			state: CutState::Idle,
			spindle_started_time: Instant::now(),
			starting_height: 0.0,
		};
		// Begin the cutting process
		ret.start_cut(cut_params);
		ret
	}

	pub fn start_cut(&mut self, params: SurfaceGrinderCutParams) {
		self.cut_params = params;
		self.starting_height = self.position_client().get_axis_position(Axis::Z);
		self.set_state(CutState::ToStartingPositionX);
	}

	// FIXME: rename to procedure_in_progress or something, to cover homing?
	pub fn cutting_in_progress(&self) -> bool {
		self.state != CutState::Idle
	}

	fn set_spindle_on(&mut self, on: bool) {
		self.send_to_motor_control(Message::SpindleControlMsgType(SpindleControlMsg{on}));
		self.spindle_started_time = Instant::now();
	}

	fn move_axis_to_extent(&mut self, axis: Axis, end: AxisEnd) {
		let position = self.work_envelope().get_extent(axis, end);
		self.move_to_position(axis, position);
	}

	fn move_to_position(&mut self, axis: Axis, position: f64) {
		self.send_to_motor_control(Message::GoToPositionMsgType(GoToPositionMsg{axis, position, speed: self.cut_params.stroke_speed}));
	}

	fn move_relative(&mut self, axis: Axis, distance: f64) {
		self.send_to_motor_control(Message::MoveAxisRelMsgType(MoveAxisRelMsg{axis, distance, speed: self.cut_params.stroke_speed}));
	}

	fn handle_movement_complete(&mut self, msg: MovementCompleteMsg) {
		if self.cutting_in_progress() {
			self.advance_state();
		}
	}

	fn advance_state(&mut self) {
		self.set_state(self.get_next_state());
	}

	fn get_next_state(&self) -> CutState {
		match self.state {
			CutState::Idle => CutState::Idle,
			CutState::ToStartingPositionX => CutState::ToStartingPositionY,
			CutState::ToStartingPositionY => CutState::SpindleSpinUp,
			CutState::SpindleSpinUp => CutState::XCut,
			CutState::XCut => CutState::XReturn,
			CutState::XReturn => {
				if self.reached_extent(Axis::Y, AxisEnd::Min) {
					CutState::YReturn
				} else {
					CutState::YOut
				}
			},
			CutState::YOut => CutState::XCut,
			CutState::YReturn => {
				// Using close_enough() because we can only move in discrete steps, so we need to check if we're within one step rather than exactly at the target
				if self.close_enough(Axis::Z, self.starting_height - self.cut_params.total_depth) {
					// Last pass completed at target depth.
					CutState::Idle
				} else {
					CutState::ZDown
				}
			},
			CutState::ZDown => CutState::XCut,
		}
	}

	fn set_state(&mut self, state: CutState) {
		println!("Setting state to {}", state);
		self.state = state;
		match self.state {
			CutState::Idle => self.stop(),
			CutState::ToStartingPositionX => self.move_axis_to_extent(Axis::X, AxisEnd::Min),
			CutState::ToStartingPositionY => self.move_axis_to_extent(Axis::Y, AxisEnd::Max),
			CutState::SpindleSpinUp => self.set_spindle_on(true),
			CutState::XCut => self.move_axis_to_extent(Axis::X, AxisEnd::Max),
			CutState::XReturn => self.move_axis_to_extent(Axis::X, AxisEnd::Min),
			CutState::YOut => self.move_relative(Axis::Y, -self.cut_params.feed_per_pass.min(self.distance_to_extent(Axis::Y, AxisEnd::Min))),
			CutState::YReturn => self.move_axis_to_extent(Axis::Y, AxisEnd::Max),
			CutState::ZDown => self.move_relative(Axis::Z, -self.cut_params.depth_of_cut.min(self.depth_remaining())),
		}
	}

	fn depth_remaining(&self) -> f64 {
		((self.starting_height - self.cut_params.total_depth) - self.position_client().get_axis_position(Axis::Z)).abs()
	}

	fn reached_extent(&self, axis: Axis, end: AxisEnd) -> bool {
		if *self.endstop_status_client().is_endstop_hit(EndstopIdentifier::new(axis, end)).unwrap_or(&false) {
			return true;
		}

		// FIXME: might be better to work in steps? Floating point is annoying...
		if end == AxisEnd::Min {
			self.position_client().get_axis_position(axis) <= self.work_envelope().get_extent(axis, end)
		} else {
			self.position_client().get_axis_position(axis) >= self.work_envelope().get_extent(axis, end)
		}
	}

	fn close_enough(&self, axis: Axis, position: f64) -> bool {
		let axis_config = self.config_client().config.motor_configs.get(&axis).unwrap();
		let current_step = axis_config.inches_to_steps(self.position_client().get_axis_position(axis));
		let target_step = axis_config.inches_to_steps(position);
		target_step == current_step
	}

	fn distance_to_extent(&self, axis: Axis, extent: AxisEnd) -> f64 {
		(self.work_envelope().get_extent(axis, extent) - self.position_client().get_axis_position(axis)).abs()
	}
}
