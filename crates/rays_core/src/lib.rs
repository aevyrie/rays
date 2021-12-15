use crossbeam_channel::{bounded, Receiver, Sender};
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
                    let r = (position[0] as f64 / (self.size[0] - 1) as f64 * 255.0) as u8;
                    let g = (position[1] as f64 / (self.size[1] - 1) as f64 * 255.0) as u8;
                    let b = 63;
                    let a = 255;
                    let color: [u8; 4] = [r, g, b, a];
                    self.sender.send(Pixel { position, color }).unwrap();
                });
        });
        self.receiver
    }
}
