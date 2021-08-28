use crate::config::ConfigClient;
use crate::endstop_checker::EndstopStatusClient;
use crate::messages::Message;
use crate::motor_control::CurrentPositionClient;

use super::OperationParameters;
use super::WorkEnvelope;

use std::sync::mpsc::Sender;



pub struct OperationControllerData {
	pub config_client: ConfigClient,
	pub endstop_status_client: EndstopStatusClient,
	pub position_client: CurrentPositionClient,
	pub motor_control_sender: Sender<Message>,
	pub work_envelope: WorkEnvelope,

	/// Flag to tell the manager to replace this controller with one created from these parameters
	pub pending_operation_params: Option<Box<dyn OperationParameters>>,
}

impl OperationControllerData {
	pub fn clone(&self) -> Self {
		Self{
			config_client: self.config_client.clone(),
			endstop_status_client: self.endstop_status_client.clone(),
			position_client: self.position_client.clone(),
			motor_control_sender: self.motor_control_sender.clone(),
			work_envelope: self.work_envelope.clone(),
			// Not cloning operation parameters because we don't need/want them for the new controller
			pending_operation_params: None,
		}
	}
}
