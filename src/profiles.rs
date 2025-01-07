use std::collections::HashSet;

pub struct ProfileLoader {
    queue: HashSet<[u8; 32]>,
    fetched: HashSet<[u8; 32]>,
}

impl ProfileLoader {
    pub fn new() -> Self {
        Self {
            queue: HashSet::new(),
            fetched: HashSet::new(),
        }
    }

    pub fn demand(&mut self, pubkey: [u8; 32]) {
        if self.fetched.contains(&pubkey) {
            return;
        }
        self.queue.insert(pubkey);
    }

    pub fn next(&mut self) -> Vec<[u8; 32]> {
        let ret: Vec<[u8; 32]> = self.queue.drain().collect();
        for p in ret.iter() {
            self.fetched.insert(*p);
        }
        ret
    }
}
