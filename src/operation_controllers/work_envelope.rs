use crate::common::Axis;
use crate::common::AxisEnd;



#[derive(Clone)]
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
