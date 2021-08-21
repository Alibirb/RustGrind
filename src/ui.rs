use crate::messages::Message;
use crate::messages::MoveAxisRelMsg;
use crate::messages::SpindleControlMsg;
use crate::surface_grinder_cut_controller::SurfaceGrinderCutParams;

use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::thread;

use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;
use rocket::State;



#[post("/", format = "json", data = "<message>")]
fn order_move_axis_rel(message: Json<MoveAxisRelMsg>, sender: State<Mutex<Sender<Message>>>) {
	sender.lock().unwrap().send(Message::MoveAxisRelMsgType(message.into_inner()));
}

#[post("/", format = "json", data = "<message>")]
fn order_spindle_power(message: Json<bool>, sender: State<Mutex<Sender<Message>>>) {
	sender.lock().unwrap().send(Message::SpindleControlMsgType(SpindleControlMsg{on: message.into_inner()}));
}

#[post("/", format = "json", data = "<message>")]
fn order_start_surface_grinder_cut(message: Json<SurfaceGrinderCutParams>, sender: State<Mutex<Sender<Message>>>) {
	sender.lock().unwrap().send(Message::StartSurfaceGrinderCutMsgType(message.into_inner()));
}

#[post("/", format = "json")]
fn order_stop(sender: State<Mutex<Sender<Message>>>) {
	sender.lock().unwrap().send(Message::StopMsgType());
}


pub fn init(sender: Sender<Message>) {
	let builder = thread::Builder::new().name("Main UI".to_string());
	builder.spawn(move || {
		let mutex = Mutex::new(sender);
		rocket::ignite()
			.manage(mutex)
			.mount("/", StaticFiles::from("html/dist/rust-grind"))
			.mount("/api/moveAxisRel", routes![order_move_axis_rel])
			.mount("/api/spindlePower", routes![order_spindle_power])
			.mount("/api/startSurfaceGrinderCut", routes![order_start_surface_grinder_cut])
			.mount("/api/stop", routes![order_stop])
			.launch();
	}).unwrap();
}
