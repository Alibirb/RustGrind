use crate::common::Axis;
use crate::common::EndstopIdentifier;
use crate::operation_controllers::SurfaceGrinderCutParams;



#[derive(Serialize, Deserialize)]
pub enum Message {
	CurrentPositionMsgType(CurrentPositionMsg),
	EndstopHitMsgType(EndstopHitMsg),
	GoToPositionMsgType(GoToPositionMsg),
	MoveAxisRelMsgType(MoveAxisRelMsg),
	MovementCompleteMsgType(MovementCompleteMsg),
	SpindleControlMsgType(SpindleControlMsg),
	StartHomingMsgType(),
	StartSurfaceGrinderCutMsgType(SurfaceGrinderCutParams),
	StopMsgType(),
}

#[derive(Copy, Clone)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct CurrentPositionMsg {
	pub x: f64,
	pub y: f64,
	pub z: f64,
}
impl CurrentPositionMsg {
	pub fn new() -> Self {
		CurrentPositionMsg{
			x: 0.0,
			y: 0.0,
			z: 0.0,
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct EndstopHitMsg {
	pub endstop: EndstopIdentifier,
	pub value: bool,
}

/**
 * Message sent to move an axis to a given position
 */
#[derive(Serialize, Deserialize)]
pub struct GoToPositionMsg {
	pub axis: Axis,
	pub position: f64,
}

/**
 * Message sent to move an axis by a relative distance
 */
#[derive(Serialize, Deserialize)]
pub struct MoveAxisRelMsg {
	pub axis: Axis,
	pub distance: f64,
}

#[derive(Copy, Clone)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct MovementCompleteMsg {
	pub axis: Axis,
	pub endstop_hit: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SpindleControlMsg {
	pub on: bool,
}
