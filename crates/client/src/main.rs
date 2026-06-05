use audio_lib::{list_devices, record_wav};

fn main() {
    // let devices = list_devices().expect("Failed to list audio devices");
    record_wav().expect("Failed to record audio");
}
