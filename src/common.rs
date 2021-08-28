use strum_macros::Display;



#[derive(Copy, Clone)]
#[derive(Display, Debug)]
#[derive(PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum Axis {
	X,
	Y,
	Z,
}


#[derive(Copy, Clone)]
#[derive(Display, Debug)]
#[derive(PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum AxisEnd {
	Min,
	Max,
}



#[derive(Copy, Clone)]
#[derive(Debug)]
#[derive(PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct EndstopIdentifier {
	pub axis: Axis,
	pub position: AxisEnd,
}

impl EndstopIdentifier {
	pub fn new(axis: Axis, position: AxisEnd) -> Self {
		EndstopIdentifier{
			axis,
			position,
		}
	}
}
