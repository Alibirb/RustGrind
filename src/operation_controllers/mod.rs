mod manual_control_controller;
mod operation_controller_data;
mod operation_controller_manager;
mod operation_controller;
mod surface_grinder_cut_controller;
mod work_envelope;

pub use self::surface_grinder_cut_controller::SurfaceGrinderCutParams;

use self::operation_controller_data::OperationControllerData;
use self::operation_controller_manager::OperationControllerManager;
use self::operation_controller::OperationController;
use self::operation_controller::OperationParameters;
use self::work_envelope::WorkEnvelope;

use crate::config::RustGrindConfig;
use crate::messages::Message;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;



pub fn init(initial_config : RustGrindConfig, receiver : Receiver<Message>, motor_control_sender: Sender<Message>) {
	let builder = thread::Builder::new().name("MainController".to_string());
	builder.spawn(move || {
		let mut main_controller = OperationControllerManager::new(initial_config, receiver, motor_control_sender);
		main_controller.run();
	}).unwrap();
}
