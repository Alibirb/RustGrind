use crate::common::Axis;
use crate::common::AxisEnd;
use crate::common::EndstopIdentifier;
use crate::config::MotorConfig;
use crate::config::RustGrindConfig;
use crate::endstop_checker::EndstopStatusClient;
use crate::messages::CurrentPositionMsg;
use crate::messages::Message;
use crate::messages::MovementCompleteMsg;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::Duration;

use stepper::{
	drivers::drv8825::DRV8825,
	motion_control, ramp_maker,
	motion_control::SoftwareMotionControl,
	traits::MotionControl,
};

use embedded_hal::digital::OutputPin;
use embedded_hal::timer::CountDown;

use linux_embedded_hal;
use linux_embedded_hal::CdevPin;
use linux_embedded_hal::gpio_cdev::Chip;
use linux_embedded_hal::gpio_cdev::LineRequestFlags;
use linux_embedded_hal::sysfs_gpio::Direction as PinDirection;
use linux_embedded_hal::SysfsPin;
use linux_embedded_hal::SysTimer;



#[derive(Clone)]
pub struct CurrentPositionClient {
	last_msg: CurrentPositionMsg,
}
impl CurrentPositionClient {
	pub fn new() -> Self {
		CurrentPositionClient{
			last_msg: CurrentPositionMsg::new(),
		}
	}

	pub fn get_position(&self) -> CurrentPositionMsg {
		self.last_msg
	}

	pub fn get_axis_position(&self, axis: Axis) -> f64 {
		match axis {
			Axis::X => self.last_msg.x,
			Axis::Y => self.last_msg.y,
			Axis::Z => self.last_msg.z,
		}
	}

	pub fn handle_message(&mut self, msg: CurrentPositionMsg) {
		self.last_msg = msg;
	}
}



type Driver = SoftwareMotionControl<
	DRV8825<
		(),
		(),
		(),
		(),
		(),
		(),
		(),
		SysfsPin,
		SysfsPin,
	>,
	SysTimer,
	ramp_maker::Trapezoidal<Num>,
	DelayToTicks,
>;

pub struct StepperMotorController  {
	// Not using "Stepper" because that interface wraps move_to_position() in a custom Future-like object with references and lifetimes and such
	// We want to just set the target position, and poll it until it reaches it or we have to stop for some reason, and the whole MoveToFuture thing requires us to hold onto a future and release it when finished, and it just makes things very complicated.
	driver: Driver,
	config: MotorConfig,
	movement_in_progress: bool,
	direction: AxisEnd,
}

impl StepperMotorController {
	pub fn new(config: MotorConfig) -> Result<StepperMotorController, linux_embedded_hal::sysfs_gpio::Error> {
		// FIXME: switch to cdev pins, and remember to run `echo PIN_NUMBER > /sys/class/gpio/unexport` for each pin to switch it away from sysfs

		let enable_pin = SysfsPin::new(config.enable_pin_number);
		match enable_pin.export() {
			Ok(()) => println!("Gpio {} exported!", enable_pin.get_pin()),
			Err(err) => println!("Gpio {} could not be exported: {}", enable_pin.get_pin(), err)
		}
		enable_pin.set_direction(PinDirection::Out)?;

		let step_pin = SysfsPin::new(config.step_pin_number);
		match step_pin.export() {
			Ok(()) => println!("Gpio {} exported!", step_pin.get_pin()),
			Err(err) => println!("Gpio {} could not be exported: {}", step_pin.get_pin(), err)
		}
		step_pin.set_direction(PinDirection::Out)?;

		let direction_pin = SysfsPin::new(config.direction_pin_number);
		match direction_pin.export() {
			Ok(()) => println!("Gpio {} exported!", direction_pin.get_pin()),
			Err(err) => println!("Gpio {} could not be exported: {}", direction_pin.get_pin(), err)
		}
		direction_pin.set_direction(PinDirection::Out)?;

		let mut timer = SysTimer::new();
		timer.try_start(Duration::from_millis(1));

		// These values assume a 1 MHz timer, but that depends on the timer you're
		// using, of course.
		let target_accel: f64 = 0.1; // steps / millisecond^2

		// We want to use the high-level motion control API (see below), but let's
		// assume the driver we use for this example doesn't provide hardware
		// support for that. Let's instantiate a motion profile from the RampMaker
		// library to provide a software fallback.
		let profile = ramp_maker::Trapezoidal::new(target_accel);

		use stepper::traits::EnableDirectionControl;
		use stepper::traits::EnableStepControl;

		Ok(
			StepperMotorController {
				driver: SoftwareMotionControl::new(
					DRV8825::new()
						.enable_direction_control(direction_pin)
						.enable_step_control(step_pin),
					timer,
					profile,
					DelayToTicks
				),
				config,
				movement_in_progress: false,
				direction: AxisEnd::Min,
			}
		)
	}

	pub fn start_move_rel(&mut self, distance: f64) -> Result<(), <Driver as MotionControl>::Error> {
		// FIXME: handle reversed motor
		// FIXME: handle too-small values: goes wrong direction if you put in min-i32
		let target_step = self.driver.current_step() + self.config.inches_to_steps(distance);

		// FIXME: handle reversed motor, i.e. opposite of what the driver reports
		if target_step < self.driver.current_step() {
			self.direction = AxisEnd::Min;
		} else {
			self.direction = AxisEnd::Max;
		}
		self.movement_in_progress = true;
		self.driver.move_to_position(self.get_steps_per_millisecond(), target_step)
	}

	pub fn start_move_to(&mut self, position: f64) -> Result<(), <Driver as MotionControl>::Error> {
		// FIXME: handle reversed motor
		// FIXME: handle too-small values: goes wrong direction if you put in min-i32
		let target_step = self.config.inches_to_steps(position);
		println!("target step is {}", target_step);

		// TODO: make common method that the various move methods will use
		// FIXME: handle reversed motor, i.e. opposite of what the driver reports
		if target_step < self.driver.current_step() {
			self.direction = AxisEnd::Min;
		} else {
			self.direction = AxisEnd::Max;
		}
		self.movement_in_progress = true;
		self.driver.move_to_position(self.get_steps_per_millisecond(), target_step)
	}

	pub fn stop_move(&mut self) -> Result<(), <Driver as MotionControl>::Error> {
		if self.movement_in_progress {
			self.start_move_rel(0.0)?;
			self.movement_in_progress = false;
		}
		Ok(())
	}

	pub fn update(&mut self) -> Result<bool, <Driver as MotionControl>::Error> {
		// Stepper driver library will try to switch direction to Backward when told to go to its current position, which means if it was going Forward and we stopped it, it will actually tell us it's still moving when we call update().
		// So if we believe we're done moving, let's not do the update.
		if self.movement_in_progress {
			self.movement_in_progress = self.driver.update()?;
		}
		Ok(self.movement_in_progress)
	}

	/// FIXME: should return Option, so we can distinguish when it's stationary.
	pub fn get_direction(&self) -> AxisEnd {
		self.direction
	}

	pub fn get_position(&self) -> f64 {
		// FIXME: handle reversed motor
		self.config.steps_to_inches(self.driver.current_step())
	}

	pub fn is_movement_in_progress(&self) -> bool {
		self.movement_in_progress
	}

	/// FIXME: should these be moved to config?
	fn get_steps_per_millisecond(&self) -> f64 {
		self.ips_to_steps_per_millisecond(self.config.default_speed_ips)
	}

	/// FIXME: should these be moved to config?
	fn ips_to_steps_per_millisecond(&self, ips: f64) -> f64 {
		ips * self.config.revs_per_inch * (self.config.steps_per_rev as f64) / 1000.0
	}
}


type Num = f64;

// Here's the converter that Stepper is going to use internally, to convert
// from the computed delay value to timer ticks. Since we chose to use timer
// ticks as the unit of time for velocity and acceleration, this conversion
// is pretty simple (and cheap).
pub struct DelayToTicks;
impl motion_control::DelayToTicks<Num> for DelayToTicks {
	type Ticks = <SysTimer as embedded_hal::timer::CountDown>::Time;
	type Error = core::convert::Infallible;

	fn delay_to_ticks(&self, delay: Num)
		-> Result<Self::Ticks, Self::Error>
	{
		let ticks = <SysTimer as embedded_hal::timer::CountDown>::Time::from_millis(delay as u64);
		Ok(ticks)
	}
}


/**
 * Object responsible for controlling the motors.
 * This object doesn't understand what we're doing, and just responds to commands to move the motors.
 */
pub struct MotorsControl {
	receiver: Receiver<Message>,
	sender: Sender<Message>,
	x_controller: StepperMotorController,
	y_controller: StepperMotorController,
	z_controller: StepperMotorController,
	endstop_status_client: EndstopStatusClient,
	last_position_msg: CurrentPositionMsg,
	spindle_pin: CdevPin,
}

impl MotorsControl {
	pub fn new(initial_config: &RustGrindConfig, receiver: Receiver<Message>, sender: Sender<Message>) -> Result<MotorsControl, linux_embedded_hal::sysfs_gpio::Error> {
		let x_config = initial_config.motor_configs.get(&Axis::X).copied().unwrap();
		let y_config = initial_config.motor_configs.get(&Axis::Y).copied().unwrap();
		let z_config = initial_config.motor_configs.get(&Axis::Z).copied().unwrap();
		let mut chip = Chip::new(initial_config.gpio_chip_name.clone()).unwrap();
		let spindle_line_handle = chip.get_line(initial_config.spindle_enable_pin).unwrap().request(LineRequestFlags::OUTPUT, 0, "spindle control").unwrap();
		let spindle_pin = CdevPin::new(spindle_line_handle).unwrap();
		Ok(MotorsControl {
			receiver,
			sender,
			x_controller: StepperMotorController::new(x_config)?,
			y_controller: StepperMotorController::new(y_config)?,
			z_controller: StepperMotorController::new(z_config)?,
			endstop_status_client: EndstopStatusClient::new(),
			last_position_msg: CurrentPositionMsg::new(),
			spindle_pin,
		})
	}

	fn get_controller_mut(&mut self, axis : Axis) -> &mut StepperMotorController {
		match axis {
			Axis::X => &mut self.x_controller,
			Axis::Y => &mut self.y_controller,
			Axis::Z => &mut self.z_controller,
		}
	}

	pub fn go_to_position(&mut self, axis : Axis, position : f64) {
		println!("Moving {:#?} to position {}", axis, position);
		// FIXME: should not move if endstop is already hit; seems like we take a step or two to recognize it.
		self.get_controller_mut(axis).start_move_to(position);
	}

	pub fn move_relative(&mut self, axis : Axis, distance : f64) {
		println!("Moving {:#?} by {}", axis, distance);
		// FIXME: should not move if endstop is already hit; seems like we take a step or two to recognize it.
		self.get_controller_mut(axis).start_move_rel(distance);
	}

	pub fn stop_all(&mut self) {
		self.x_controller.stop_move();
		self.y_controller.stop_move();
		self.z_controller.stop_move();
		self.set_spindle_on(false);
	}

	fn set_spindle_on(&mut self, on: bool) -> Result<(), <CdevPin as embedded_hal::digital::OutputPin>::Error> {
		if on {
			self.spindle_pin.try_set_high()
		} else {
			self.spindle_pin.try_set_low()
		}
	}

	// TODO: take in a string with a reason for the shutdown
	fn shutdown(&mut self) {
		println!("Shutting down motor control");
		self.stop_all();
	}

	fn handle_message(&mut self, msg : Message) {
		match msg {
			Message::EndstopHitMsgType(eh_msg) => self.endstop_status_client.process_message(eh_msg),
			Message::GoToPositionMsgType(gtp_msg) => self.go_to_position(gtp_msg.axis, gtp_msg.position),
			Message::MoveAxisRelMsgType(mar_msg) => self.move_relative(mar_msg.axis, mar_msg.distance),
			Message::SpindleControlMsgType(sc_msg) => self.set_spindle_on(sc_msg.on).unwrap(),
			Message::StopMsgType() => self.stop_all(),

			Message::CurrentPositionMsgType(_) => {},
			Message::MovementCompleteMsgType(_) => {},
			Message::StartSurfaceGrinderCutMsgType(_) => {},
		};
	}

	fn check_endstops(&mut self) {
		self.check_endstops_for_axis(Axis::X);
		self.check_endstops_for_axis(Axis::Y);
		self.check_endstops_for_axis(Axis::Z);
	}

	fn check_endstops_for_axis(&mut self, axis: Axis) {
		let current_direction = self.get_controller_mut(axis).get_direction();
		let endstop_hit = self.endstop_status_client.is_endstop_hit(EndstopIdentifier::new(axis, current_direction)).copied();
		if endstop_hit.contains(&true) {
			if self.get_controller_mut(axis).is_movement_in_progress() {
				println!("Hit endstop {} {}", axis, current_direction);

				self.get_controller_mut(axis).stop_move();

				println!("Endstop hit; sending MovementComplete for {}", axis);
				let msg = MovementCompleteMsg{ axis, endstop_hit: true};
				self.sender.send(Message::MovementCompleteMsgType(msg));
			}
		}
	}

	fn send_position_update(&mut self) {
		let msg = CurrentPositionMsg{
			x: self.x_controller.get_position(),
			y: self.y_controller.get_position(),
			z: self.z_controller.get_position(),
		};
		if msg != self.last_position_msg {
			self.sender.send(Message::CurrentPositionMsgType(msg));
			self.last_position_msg = msg;
		}
	}

	fn update_motor(&mut self, axis: Axis) {
		let prev_movement_in_progress = self.get_controller_mut(axis).is_movement_in_progress();
		match self.get_controller_mut(axis).update() {
			Ok(ongoing) => {
				if !ongoing && prev_movement_in_progress {
					println!("Sending MovementComplete for {}", axis);
					let msg = MovementCompleteMsg{ axis, endstop_hit: false};
					self.sender.send(Message::MovementCompleteMsgType(msg));
				}
			},
			Err(error) => println!("Encountered error updating axis {}, error is {:?}", axis, error),
		}
	}

	fn update_controllers(&mut self) {
		self.update_motor(Axis::X);
		self.update_motor(Axis::Y);
		self.update_motor(Axis::Z);
	}

	pub fn run(&mut self) -> ! {
		loop {
			// Check for messages
			loop {
				match self.receiver.try_recv() {
					Ok(msg) => self.handle_message(msg),
					Err(TryRecvError::Empty) => break,
					// If nothing is connected to send us messages, then nothing is in control of the motors, so shut down immediately.
					// There is no way for anything to get connected again, either, so this whole thread is trash.
					Err(TryRecvError::Disconnected) => self.shutdown(),
				}
			}
			
			self.check_endstops();
			self.update_controllers();
			self.send_position_update();

			// TODO: should probably sleep for like a nanosecond or something so we're not always busy-waiting
			// Or perhaps use yield_now() instead? I don't know...
		}
	}
}


pub fn init(initial_config : RustGrindConfig, receiver : Receiver<Message>, sender: Sender<Message>) {
	let builder = thread::Builder::new().name("MotorControl".to_string());
	builder.spawn(move || {
		let mut main_motor_controller = MotorsControl::new(&initial_config, receiver, sender).unwrap();
		main_motor_controller.run();
	}).unwrap();
}
