pub trait Memento {
    fn recall(&mut self, block: u64, buf: &mut [u8; 512]);
    fn remember(&mut self, block: u64, buf: &[u8; 512]);
}