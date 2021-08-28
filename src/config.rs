use crate::common::Axis;
use crate::common::AxisEnd;
use crate::common::EndstopIdentifier;
use crate::pins;

use std::collections::HashMap;
use std::error;
use std::fs::File;
use std::io::BufReader;

use serde_yaml;



const CONFIG_FILE_PATH : &str = "/home/pi/rust_grind.yaml";

#[derive(Copy, Clone)]
#[derive(Serialize, Deserialize)]
pub struct MotorConfig {
	pub steps_per_rev: i32,
	pub revs_per_inch: f64,
	pub reversed: bool,
	pub default_speed_ips: f64,

	pub enable_pin_number: u64,
	pub step_pin_number: u64,
	pub direction_pin_number: u64,
}
impl MotorConfig {
	pub fn inches_to_steps(&self, inches: f64) -> i32 {
		(inches * self.revs_per_inch * (self.steps_per_rev as f64)) as i32
	}

	pub fn steps_to_inches(&self, steps: i32) -> f64 {
		(steps as f64) / (self.revs_per_inch * (self.steps_per_rev as f64))
	}
}

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct RustGrindConfig {
	pub motor_configs: HashMap<Axis, MotorConfig>,
	// FIXME: needs to specify normally open or closed
	pub endstop_config: HashMap<EndstopIdentifier, u32>,
	pub gpio_chip_name: String,
	pub spindle_enable_pin: u32,
}


/// TODO: need to synchronize config
#[derive(Clone)]
pub struct ConfigClient {
	pub config: RustGrindConfig,
}

impl ConfigClient {
	pub fn new(config: RustGrindConfig) -> Self {
		ConfigClient {
			config,
		}
	}
}



pub struct ConfigManager {
	config: RustGrindConfig,
}

impl ConfigManager {
	pub fn new() -> ConfigManager {
		let mut ret = ConfigManager {
			config: RustGrindConfig {
				motor_configs: HashMap::new(),
				endstop_config: HashMap::new(),
				gpio_chip_name: "/dev/gpiochip0".to_string(),
				spindle_enable_pin: pins::SPINDLE_PIN_NUMBER,
			}
		};
		ret.config.motor_configs.insert(Axis::X, MotorConfig {
			steps_per_rev: 200,
			revs_per_inch: 1.0,
			reversed: false,
			default_speed_ips: 1.0,
			enable_pin_number: pins::X_ENABLE_PIN_NUMBER,
			step_pin_number: pins::X_STEP_PIN_NUMBER,
			direction_pin_number: pins::X_DIRECTION_PIN_NUMBER,
		});
		ret.config.motor_configs.insert(Axis::Y, MotorConfig {
			steps_per_rev: 200,
			revs_per_inch: 1.0,
			reversed: false,
			default_speed_ips: 1.0,
			enable_pin_number: pins::Y_ENABLE_PIN_NUMBER,
			step_pin_number: pins::Y_STEP_PIN_NUMBER,
			direction_pin_number: pins::Y_DIRECTION_PIN_NUMBER,
		});
		ret.config.motor_configs.insert(Axis::Z, MotorConfig {
			steps_per_rev: 200,
			revs_per_inch: 1.0,
			reversed: false,
			default_speed_ips: 1.0,
			enable_pin_number: pins::Z_ENABLE_PIN_NUMBER,
			step_pin_number: pins::Z_STEP_PIN_NUMBER,
			direction_pin_number: pins::Z_DIRECTION_PIN_NUMBER,
		});

		ret.config.endstop_config.insert(EndstopIdentifier{axis: Axis::X, position: AxisEnd::Min}, pins::X_MIN_ENDSTOP_PIN_NUMBER);
		ret.config.endstop_config.insert(EndstopIdentifier{axis: Axis::X, position: AxisEnd::Max}, pins::X_MAX_ENDSTOP_PIN_NUMBER);
		ret.config.endstop_config.insert(EndstopIdentifier{axis: Axis::Y, position: AxisEnd::Min}, pins::Y_MIN_ENDSTOP_PIN_NUMBER);
		ret.config.endstop_config.insert(EndstopIdentifier{axis: Axis::Y, position: AxisEnd::Max}, pins::Y_MAX_ENDSTOP_PIN_NUMBER);
		ret.config.endstop_config.insert(EndstopIdentifier{axis: Axis::Z, position: AxisEnd::Max}, pins::Z_MAX_ENDSTOP_PIN_NUMBER);

		ret
	}

	// FIXME: may not need to return result, just handle internally?
	pub fn read_config_file(&mut self) -> Result<(), Box<dyn error::Error>> {
		let file = File::open(CONFIG_FILE_PATH)?;
		let buf_reader = BufReader::new(file);
		self.config = serde_yaml::from_reader(buf_reader)?;
		Ok(())
	}

	pub fn write_config_file(&self) {
		let file = File::create(CONFIG_FILE_PATH).unwrap();
		serde_yaml::to_writer(file, &self.config).unwrap();
	}

	pub fn get_config(&self) -> &RustGrindConfig {
		&self.config
	}
}
