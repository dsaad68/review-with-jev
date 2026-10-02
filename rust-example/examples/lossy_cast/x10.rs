use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::thread;

const CHUNK: i64 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ChunkPos {
    x: i32,
    z: i32,
}

#[derive(Debug)]
enum WorldError {
    OutOfBounds { x: i64, z: i64 },
}

impl fmt::Display for WorldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorldError::OutOfBounds { x, z } => write!(f, "({x}, {z}) is outside the world"),
        }
    }
}

impl Error for WorldError {}

fn chunk_of(x: i64, z: i64) -> Result<(ChunkPos, usize), WorldError> {
    let oob = || WorldError::OutOfBounds { x, z };
    let cx = i32::try_from(x.div_euclid(CHUNK)).map_err(|_| oob())?;
    let cz = i32::try_from(z.div_euclid(CHUNK)).map_err(|_| oob())?;
    let local = usize::try_from(z.rem_euclid(CHUNK) * CHUNK + x.rem_euclid(CHUNK))
        .map_err(|_| oob())?;
    Ok((ChunkPos { x: cx, z: cz }, local))
}

trait Terrain: Sync {
    fn height(&self, x: i64, z: i64) -> i64;
}

struct Hills {
    seed: u32,
}

impl Terrain for Hills {
    fn height(&self, x: i64, z: i64) -> i64 {
        let mixed = x.wrapping_mul(73_856_093) ^ z.wrapping_mul(19_349_663) ^ i64::from(self.seed);
        mixed.rem_euclid(400) - 80
    }
}

struct Chunk {
    heights: Vec<u8>,
}

fn generate<T: Terrain>(terrain: &T, pos: ChunkPos) -> Chunk {
    let base_x = i64::from(pos.x) * CHUNK;
    let base_z = i64::from(pos.z) * CHUNK;
    let mut heights = Vec::with_capacity(256);
    for lz in 0..CHUNK {
        for lx in 0..CHUNK {
            let h = terrain.height(base_x + lx, base_z + lz);
            heights.push(h.clamp(0, i64::from(u8::MAX)) as u8);
        }
    }
    Chunk { heights }
}

struct World<'t, T: Terrain> {
    terrain: &'t T,
    chunks: HashMap<ChunkPos, Chunk>,
}

impl<'t, T: Terrain> World<'t, T> {
    fn new(terrain: &'t T) -> Self {
        World {
            terrain,
            chunks: HashMap::new(),
        }
    }

    fn load_area(&mut self, positions: &[ChunkPos], workers: usize) {
        let per = positions.len().div_ceil(workers.max(1)).max(1);
        let terrain = self.terrain;
        let generated: Vec<(ChunkPos, Chunk)> = thread::scope(|s| {
            let handles: Vec<_> = positions
                .chunks(per)
                .map(|batch| {
                    s.spawn(move || {
                        batch
                            .iter()
                            .map(|&p| (p, generate(terrain, p)))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            handles
                .into_iter()
                .flat_map(|h| h.join().unwrap())
                .collect()
        });
        self.chunks.extend(generated);
    }

    fn height_at(&self, x: i64, z: i64) -> Result<Option<u8>, WorldError> {
        let (pos, local) = chunk_of(x, z)?;
        Ok(self.chunks.get(&pos).and_then(|c| c.heights.get(local).copied()))
    }

    fn mean_height(&self) -> Option<u8> {
        let (total, count) = self
            .chunks
            .values()
            .flat_map(|c| c.heights.iter())
            .fold((0u64, 0u64), |(t, n), &h| (t + u64::from(h), n + 1));
        if count == 0 {
            return None;
        }
        u8::try_from(total / count).ok()
    }
}

fn main() {
    let terrain = Hills { seed: 4242 };
    let mut world = World::new(&terrain);
    let positions: Vec<ChunkPos> = (-2..2)
        .flat_map(|x| (-2..2).map(move |z| ChunkPos { x, z }))
        .collect();
    world.load_area(&positions, 4);
    println!("loaded {} chunks", world.chunks.len());
    for (x, z) in [(0, 0), (-17, 5), (31, -32), (500, 500), (i64::MAX, 0)] {
        match world.height_at(x, z) {
            Ok(Some(h)) => println!("height at ({x}, {z}) = {h}"),
            Ok(None) => println!("({x}, {z}) is not loaded"),
            Err(e) => println!("error: {e}"),
        }
    }
    println!("mean height {:?}", world.mean_height());
}
