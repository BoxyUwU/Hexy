use bevy_inspector_egui::Inspectable;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Inspectable)]
pub struct HexPos {
    pub q: i32,
    pub r: i32,
}

// dont `derive(Default)` the `tiles` field will have length 0
// Not Inspectable because of Box<[T]>
#[derive(Debug)]
pub struct HexMap<T> {
    width: usize,
    height: usize,
    tiles: Box<[T]>,
}

impl<T> HexMap<T> {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn new(width: usize, height: usize, tiles: impl IntoIterator<Item = T>) -> Self {
        let mut tiles_iter = tiles.into_iter();
        let tiles = (&mut tiles_iter).take(width * height).collect::<Box<[T]>>();
        assert!(matches!(tiles_iter.next(), None));
        assert_eq!(tiles.len(), width * height);
        Self {
            width,
            height,
            tiles,
        }
    }

    pub fn get(&self, pos: HexPos) -> &T {
        assert!(pos.q >= 0);
        assert!(pos.r >= 0);
        assert!((pos.q as usize) < self.width);
        assert!((pos.r as usize) < self.height);

        let foo = pos.q as usize + ((pos.r as usize) * self.width);
        &self.tiles[foo]
    }

    pub fn get_mut(&mut self, pos: HexPos) -> &mut T {
        assert!(pos.q >= 0);
        assert!(pos.r >= 0);
        assert!((pos.q as usize) < self.width);
        assert!((pos.r as usize) < self.height);

        let foo = pos.q as usize + ((pos.r as usize) * self.width);
        &mut self.tiles[foo]
    }
}

pub fn wrap_hex_pos(pos: HexPos, map_width: u32, map_height: u32) -> HexPos {
    let q = if pos.q >= map_width as i32 {
        pos.q % map_width as i32
    } else if pos.q < 0 {
        (map_width as i32 - 1) - ((pos.q.abs() - 1) % map_width as i32)
    } else {
        pos.q
    };
    let r = if pos.r >= map_height as i32 {
        pos.r % map_height as i32
    } else if pos.r < 0 {
        (map_height as i32 - 1) - ((pos.r.abs() - 1) % map_height as i32)
    } else {
        pos.r
    };
    HexPos { q, r }
}
