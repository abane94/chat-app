use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("No input device");
    let output_device = host.default_output_device().expect("No output device");

    let config = input_device.default_input_config()?;
    let err_fn = |err| eprintln!("An error occurred: {}", err);

    // Create SPSC ring buffer (adjust capacity based on desired latency)
    let ring = HeapRb::<f32>::new(2048);
    let (mut producer, mut consumer) = ring.split();

    // 1. Build & Play Input Stream
    let input_stream = input_device.build_input_stream(
        &config.config(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let _ = producer.push_slice(data); // Send input to buffer
        },
        err_fn.clone(),
        None,
    )?;
    input_stream.play()?;

    // 2. Build & Play Output Stream
    let output_stream = output_device.build_output_stream(
        &config.config(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Read from buffer or pad with silence if buffer is empty
            let read = consumer.pop_slice(data);
            for sample in &mut data[read..] {
                *sample = 0.0u32.to_sample(); // Pad with silence
            }
        },
        err_fn,
        None,
    )?;
    output_stream.play()?;

    // Keep streams alive
    std::thread::sleep(std::time::Duration::from_secs(60));
    Ok(())
}
