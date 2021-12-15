use crossbeam_channel::{bounded, Receiver, Sender};
use rand::prelude::*;
use rayon::prelude::*;

pub struct PathTracer {
    size: [usize; 2],
    sender: Sender<Pixel>,
    receiver: Receiver<Pixel>,
}

pub struct Pixel {
    pub position: [usize; 2],
    pub color: [u8; 4],
}

impl PathTracer {
    pub fn build(size: [usize; 2]) -> PathTracer {
        let (sender, receiver) = bounded((size[0] * size[1]) as usize);
        PathTracer {
            size,
            sender,
            receiver,
        }
    }

    pub fn run(self) -> Receiver<Pixel> {
        std::thread::spawn(move || {
            (0..self.size[0] * self.size[1])
                .par_bridge()
                .for_each(|index| {
                    let position = [index % self.size[0], index / self.size[0]];
                    let mut color: [u8; 4] = [0; 4];
                    rand::thread_rng().fill_bytes(&mut color);
                    self.sender.send(Pixel { position, color }).unwrap();
                });
        });
        self.receiver
    }
}
