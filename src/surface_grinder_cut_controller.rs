use crate::common::Axis;
use crate::common::AxisEnd;
use crate::common::EndstopIdentifier;
use crate::config::ConfigClient;
use crate::config::RustGrindConfig;
use crate::endstop_checker::EndstopStatusClient;
use crate::messages::GoToPositionMsg;
use crate::messages::Message;
use crate::messages::MoveAxisRelMsg;
use crate::messages::MovementCompleteMsg;
use crate::messages::SpindleControlMsg;
use crate::motor_control::CurrentPositionClient;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use strum_macros::Display;



pub struct WorkEnvelope {
	pub min_x: f64,
	pub min_y: f64,
	pub min_z: f64,
	pub max_x: f64,
	pub max_y: f64,
	pub max_z: f64,
}

impl WorkEnvelope {
	pub fn new() -> Self {
		WorkEnvelope{
			min_x: -256.0,
			min_y: -256.0,
			min_z: 0.0,
			max_x: 256.0,
			max_y: 256.0,
			max_z: 0.0,
		}
	}

	pub fn get_extent(&self, axis: Axis, end: AxisEnd) -> f64 {
		match (axis, end) {
			(Axis::X, AxisEnd::Min) => self.min_x,
			(Axis::Y, AxisEnd::Min) => self.min_y,
			(Axis::Z, AxisEnd::Min) => self.min_z,
			(Axis::X, AxisEnd::Max) => self.max_x,
			(Axis::Y, AxisEnd::Max) => self.max_y,
			(Axis::Z, AxisEnd::Max) => self.max_z,
		}
	}
}

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
pub struct SurfaceGrinderCutController {
	config_client: ConfigClient,
	cut_params: SurfaceGrinderCutParams,
	position_client: CurrentPositionClient,
	endstop_status_client: EndstopStatusClient,
	receiver: Receiver<Message>,
	motor_control_sender: Sender<Message>,
	state: CutState,
	work_envelope: WorkEnvelope,
	spindle_started_time: Instant,
	starting_height: f64,
}

impl SurfaceGrinderCutController {
	pub fn new(config: RustGrindConfig, receiver: Receiver<Message>, motor_control_sender: Sender<Message>) -> Self {
		SurfaceGrinderCutController {
			config_client: ConfigClient::new(config),
			cut_params: SurfaceGrinderCutParams::new(),
			position_client: CurrentPositionClient::new(),
			endstop_status_client: EndstopStatusClient::new(),
			receiver,
			motor_control_sender,
			state: CutState::Idle,
			work_envelope: WorkEnvelope::new(),
			spindle_started_time: Instant::now(),
			starting_height: 0.0,
		}
	}

	pub fn start_cut(&mut self, params: SurfaceGrinderCutParams) {
		self.cut_params = params;
		self.starting_height = self.position_client.get_axis_position(Axis::Z);
		self.set_state(CutState::ToStartingPositionX);
	}

	pub fn start_home(&mut self) {
		// TODO
	}

	pub fn stop(&mut self) {
		println!("Stopping all movement");
		self.motor_control_sender.send(Message::StopMsgType());
		// Set state to Idle.
		// set_state(CutState::Idle) will actually call this function, so set the state directly here without calling it.
		// (We set the state here in case it wasn't set_state() that called this function and the state is still set to something else)
		self.state = CutState::Idle;
	}

	// FIXME: rename to procedure_in_progress or something, to cover homing?
	pub fn cutting_in_progress(&self) -> bool {
		self.state != CutState::Idle
	}

	fn set_spindle_on(&mut self, on: bool) {
		self.motor_control_sender.send(Message::SpindleControlMsgType(SpindleControlMsg{on}));
		self.spindle_started_time = Instant::now();
	}

	fn move_axis_to_extent(&mut self, axis: Axis, end: AxisEnd) {
		let position = self.work_envelope.get_extent(axis, end);
		self.move_to_position(axis, position);
	}

	fn move_to_position(&mut self, axis: Axis, position: f64) {
		self.motor_control_sender.send(Message::GoToPositionMsgType(GoToPositionMsg{axis, position}));
	}

	fn move_relative(&mut self, axis: Axis, distance: f64) {
		self.motor_control_sender.send(Message::MoveAxisRelMsgType(MoveAxisRelMsg{axis, distance}));
	}

	fn forward_to_motor_control(&self, msg: Message) {
		if !self.cutting_in_progress() {
			self.motor_control_sender.send(msg);
		}
	}

	fn handle_message(&mut self, msg : Message) {
		match msg {
			Message::CurrentPositionMsgType(cp_msg) => self.position_client.handle_message(cp_msg),
			Message::EndstopHitMsgType(eh_msg) => self.endstop_status_client.process_message(eh_msg),
			Message::GoToPositionMsgType(_) => self.forward_to_motor_control(msg),
			Message::MoveAxisRelMsgType(_) => self.forward_to_motor_control(msg),
			Message::MovementCompleteMsgType(mc_msg) => self.handle_movement_complete(mc_msg),
			Message::SpindleControlMsgType(_) => self.forward_to_motor_control(msg),
			Message::StartSurfaceGrinderCutMsgType(cut_params) => self.start_cut(cut_params),
			Message::StopMsgType() => self.stop(),
		};
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
		((self.starting_height - self.cut_params.total_depth) - self.position_client.get_axis_position(Axis::Z)).abs()
	}

	fn reached_extent(&self, axis: Axis, end: AxisEnd) -> bool {
		if *self.endstop_status_client.is_endstop_hit(EndstopIdentifier::new(axis, end)).unwrap_or(&false) {
			return true;
		}

		// FIXME: might be better to work in steps? Floating point is annoying...
		if end == AxisEnd::Min {
			self.position_client.get_axis_position(axis) <= self.work_envelope.get_extent(axis, end)
		} else {
			self.position_client.get_axis_position(axis) >= self.work_envelope.get_extent(axis, end)
		}
	}

	fn close_enough(&self, axis: Axis, position: f64) -> bool {
		let axis_config = self.config_client.config.motor_configs.get(&axis).unwrap();
		let current_step = axis_config.inches_to_steps(self.position_client.get_axis_position(axis));
		let target_step = axis_config.inches_to_steps(position);
		target_step == current_step
	}

	fn distance_to_extent(&self, axis: Axis, extent: AxisEnd) -> f64 {
		(self.work_envelope.get_extent(axis, extent) - self.position_client.get_axis_position(axis)).abs()
	}

	fn shutdown(&mut self) {
		// TODO: handle appriopriately
		self.stop();
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

			if
				self.state == CutState::SpindleSpinUp
				&& self.spindle_started_time.elapsed() >= Duration::from_secs(3)
			{
				self.advance_state();
			}

			std::thread::sleep(Duration::from_nanos(5));
		}
	}
}


pub fn init(initial_config : RustGrindConfig, receiver : Receiver<Message>, motor_control_sender: Sender<Message>) {
	let builder = thread::Builder::new().name("MainController".to_string());
	builder.spawn(move || {
		let mut main_controller = SurfaceGrinderCutController::new(initial_config, receiver, motor_control_sender);
		main_controller.run();
	}).unwrap();
}
