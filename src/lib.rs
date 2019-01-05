// Re-exports
pub mod client;
mod error;
pub mod torrent;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
