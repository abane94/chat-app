use cpal::{self, traits::HostTrait};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use cpal::traits::DeviceTrait;

use ringbuf::wrap::caching::Caching;
use std::sync::Arc;
use ringbuf::rb::shared::SharedRb;
use ringbuf::storage::Heap;

use cpal::Sample;

type RingBuffProducer = Caching<Arc<SharedRb<Heap<f32>>>, true, false>;
type RingBuffConsumer = Caching<Arc<SharedRb<Heap<f32>>>, false, true>;

fn test() {
    let host = cpal::default_host();
    let input_dev = host.default_input_device();
}

pub struct AudioManager {
    input_dev: Option<cpal::Device>,
    output_dev: Option<cpal::Device>,

    // Create SPSC ring buffer (adjust capacity based on desired latency)
    // local_ring: HeapRb<f32>,
    // remote_ring: HeapRb<f32>,

    //local_ring
    mic_input_producer: RingBuffProducer,
    pub local_audio: RingBuffConsumer,

    // remote ring
    pub remote_audio: RingBuffProducer,
    speaker_output: RingBuffConsumer,
}

// todo: publec get method as this will be a singleton
impl AudioManager {

    fn new() -> AudioManager {
        let host = cpal::default_host();
        let input_dev = host.default_input_device().expect("No input device");
        let output_dev = host.default_output_device().expect("No output device");

        let config = input_dev.default_input_config().expect("Error retrieving default input config");
        let err_fn = |err| eprintln!("An error occurred: {}", err);

        // buffers
        // Caching<Arc<SharedRb<Heap<f32>>>, true, false>
        // Caching<Arc<SharedRb<Heap<f32>>>
        let local_ring = HeapRb::<f32>::new(2048);
        let (mut mic_input_producer, mut local_audio) = local_ring.split();

        let remote_ring = HeapRb::<f32>::new(2048);
        let (mut remote_audio, mut speaker_output) = remote_ring.split();


        // streams
        let input_stream = input_dev.build_input_stream(
            &config.config(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let _ = mic_input_producer.push_slice(data); // Send input to buffer
            },
            err_fn.clone(),
            None,
        ).expect("Error building input stream");
        // input_stream.play()?;


        let output_stream = output_dev.build_output_stream(
            &config.config(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Read from buffer or pad with silence if buffer is empty
                let read = speaker_output.pop_slice(data);
                for sample in &mut data[read..] {
                    *sample = 0.0f32.to_sample(); // Pad with silence
                }
            },
            err_fn,
            None,
        ).expect("error building output stream");
        // output_stream.play()?;

        AudioManager {
            input_dev: Some(input_dev),
            output_dev: Some(output_dev),
            // local_ring,
            // remote_ring,
            mic_input_producer,
            local_audio,
            remote_audio,
            speaker_output
        }
    }
}
