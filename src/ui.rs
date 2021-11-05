use crate::messages::Message;
use crate::messages::MoveAxisRelMsg;
use crate::messages::SpindleControlMsg;
use crate::operation_controllers::SurfaceGrinderCutParams;

use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::thread;

use rocket_contrib::json::Json;
use rocket::response::NamedFile;
use rocket::State;



#[post("/", format = "json", data = "<message>")]
fn order_move_axis_rel(message: Json<MoveAxisRelMsg>, sender: State<Mutex<Sender<Message>>>) {
	sender.lock().unwrap().send(Message::MoveAxisRelMsgType(message.into_inner()));
}

#[post("/", format = "json", data = "<message>")]
fn order_spindle_power(message: Json<bool>, sender: State<Mutex<Sender<Message>>>) {
	sender.lock().unwrap().send(Message::SpindleControlMsgType(SpindleControlMsg{on: message.into_inner()}));
}

#[post("/", format = "json")]
fn order_start_homing(sender: State<Mutex<Sender<Message>>>) {
	sender.lock().unwrap().send(Message::StartHomingMsgType());
}

#[post("/", format = "json", data = "<message>")]
fn order_start_surface_grinder_cut(message: Json<SurfaceGrinderCutParams>, sender: State<Mutex<Sender<Message>>>) {
	sender.lock().unwrap().send(Message::StartSurfaceGrinderCutMsgType(message.into_inner()));
}

#[post("/", format = "json")]
fn order_stop(sender: State<Mutex<Sender<Message>>>) {
	sender.lock().unwrap().send(Message::StopMsgType());
}


#[get("/<file..>", rank = 2)]
pub fn fallback_url(file: PathBuf) -> Option<NamedFile> {
	// Serve matching file if it exists, or serve index page if it doesn't.
	// TODO: should check if a file extension was provided, and only serve the index if there's no extension, so missing assets return 404 properly
	NamedFile::open(Path::new("html/dist/rust-grind/").join(file))
		.ok().or_else(|| NamedFile::open(Path::new("html/dist/rust-grind/index.html")).ok())
}

/// Handle root path
#[get("/", rank = 1)]
pub fn index() -> Option<NamedFile> {
	NamedFile::open(Path::new("html/dist/rust-grind/index.html")).ok()
}


pub fn init(sender: Sender<Message>) {
	let builder = thread::Builder::new().name("Main UI".to_string());
	builder.spawn(move || {
		let mutex = Mutex::new(sender);
		rocket::ignite()
			.manage(mutex)
			.mount("/", routes![fallback_url, index])
			.mount("/api/moveAxisRel", routes![order_move_axis_rel])
			.mount("/api/spindlePower", routes![order_spindle_power])
			.mount("/api/startHoming", routes![order_start_homing])
			.mount("/api/startSurfaceGrinderCut", routes![order_start_surface_grinder_cut])
			.mount("/api/stop", routes![order_stop])
			.launch();
	}).unwrap();
}
