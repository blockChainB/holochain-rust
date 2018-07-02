use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use chain::entry::Entry;
use chain::SourceChain;

// @TODO - serialize properties as defined in HeadersEntrySchema from golang alpha 1
// @see https://github.com/holochain/holochain-proto/blob/4d1b8c8a926e79dfe8deaa7d759f930b66a5314f/entry_headers.go#L7
// @see https://github.com/holochain/holochain-rust/issues/75
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Header {
    /// the type of this entry
    /// system types may have associated "subconscious" behavior
    entry_type: String,
    /// ISO8601 time stamp
    time: String,
    /// link to the immediately preceding header, None is valid only for genesis
    next: Option<u64>,
    /// mandatory link to the entry for this header
    entry: u64,
    /// link to the most recent header of the same type, None is valid only for the first of type
    type_next: Option<u64>,
    /// agent's cryptographic signature
    signature: String,
}

impl Hash for Header {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entry_type.hash(state);
        self.time.hash(state);
        self.next.hash(state);
        self.entry.hash(state);
        self.type_next.hash(state);
        self.signature.hash(state);
    }
}

impl PartialEq for Header {
    fn eq(&self, other: &Header) -> bool {
        self.hash() == other.hash()
    }
}

impl Header {
    /// build a new Header from a chain, entry type and entry.
    /// a Header is immutable, but the chain is mutable if chain.push() is used.
    /// this means that a header becomes invalid and useless as soon as the chain is mutated
    /// the only valid usage of a header is to immediately push it onto a chain in a Pair.
    /// normally (outside unit tests) the generation of valid headers is internal to the
    /// chain::SourceChain trait and should not need to be handled manually
    /// @see chain::pair::Pair
    /// @see chain::entry::Entry
    pub fn new<'de, C: SourceChain<'de>>(chain: &C, entry: &Entry) -> Header {
        Header {
            entry_type: entry.entry_type().clone(),
            // @TODO implement timestamps
            // https://github.com/holochain/holochain-rust/issues/70
            time: String::new(),
            next: chain.top().and_then(|p| Some(p.header().hash())),
            entry: entry.hash(),
            type_next: chain
                .top_type(&entry.entry_type())
                .and_then(|p| Some(p.header().hash())),
            // @TODO implement signatures
            // https://github.com/holochain/holochain-rust/issues/71
            signature: String::new(),
        }
    }

    /// entry_type getter
    pub fn entry_type(&self) -> String {
        self.entry_type.clone()
    }

    /// time getter
    pub fn time(&self) -> String {
        self.time.clone()
    }

    /// next getter
    pub fn next(&self) -> Option<u64> {
        self.next
    }

    /// entry getter
    pub fn entry(&self) -> u64 {
        self.entry
    }

    /// type_next getter
    pub fn type_next(&self) -> Option<u64> {
        self.type_next
    }

    /// signature getter
    pub fn signature(&self) -> String {
        self.signature.clone()
    }

    /// hashes the header
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&self, &mut hasher);
        hasher.finish()
    }

    /// returns true if the header is valid
    pub fn validate(&self) -> bool {
        // always valid iff immutable and new() enforces validity
        true
    }
}

#[cfg(test)]
mod tests {
    use chain::SourceChain;
    use chain::entry::Entry;
    use chain::header::Header;
    use chain::memory::MemChain;

    #[test]
    /// tests for Header::new()
    fn new() {
        let chain = MemChain::new();
        let t = "type";
        let e = Entry::new(t, "foo");
        let h = Header::new(&chain, &e);

        assert_eq!(h.entry(), e.hash());
        assert_eq!(h.next(), None);
        assert_ne!(h.hash(), 0);
        assert!(h.validate());
    }

    #[test]
    /// tests for header.entry_type()
    fn entry_type() {
        let chain = MemChain::new();
        let t = "foo";
        let e = Entry::new(t, "");
        let h = Header::new(&chain, &e);

        assert_eq!(h.entry_type(), "foo");
    }

    #[test]
    /// tests for header.time()
    fn time() {
        let chain = MemChain::new();
        let t = "foo";
        let e = Entry::new(t, "");
        let h = Header::new(&chain, &e);

        assert_eq!(h.time(), "");
    }

    #[test]
    /// tests for header.next()
    fn next() {
        let mut chain = MemChain::new();
        let t = "foo";

        // first header is genesis so next should be None
        let e1 = Entry::new(t, "");
        let p1 = chain.push(&e1);
        let h1 = p1.header();

        assert_eq!(h1.next(), None);

        // second header next should be first header hash
        let e2 = Entry::new(t, "foo");
        let p2 = chain.push(&e2);
        let h2 = p2.header();

        assert_eq!(h2.next(), Some(h1.hash()));
    }

    #[test]
    /// tests for header.entry()
    fn entry() {
        let chain = MemChain::new();
        let t = "foo";

        // header for an entry should contain the entry hash under entry()
        let e = Entry::new(t, "");
        let h = Header::new(&chain, &e);

        assert_eq!(h.entry(), e.hash());
    }

    #[test]
    /// tests for header.type_next()
    fn type_next() {
        let mut chain = MemChain::new();
        let t1 = "foo";
        let t2 = "bar";

        // first header is genesis so next should be None
        let e1 = Entry::new(t1, "");
        let p1 = chain.push(&e1);
        let h1 = p1.header();

        assert_eq!(h1.type_next(), None);

        // second header is a different type so next should be None
        let e2 = Entry::new(t2, "");
        let p2 = chain.push(&e2);
        let h2 = p2.header();

        assert_eq!(h2.type_next(), None);

        // third header is same type as first header so next should be first header hash
        let e3 = Entry::new(t1, "");
        let p3 = chain.push(&e3);
        let h3 = p3.header();

        assert_eq!(h3.type_next(), Some(h1.hash()));
    }

    #[test]
    /// tests for header.signature()
    fn signature() {
        let chain = MemChain::new();
        let t = "foo";

        let e = Entry::new(t, "");
        let h = Header::new(&chain, &e);

        assert_eq!("", h.signature());
    }

    #[test]
    /// test header.hash() against a known value
    fn hash_known() {
        let chain = MemChain::new();
        let t = "foo";

        // check a known hash
        let e = Entry::new(t, "");
        let h = Header::new(&chain, &e);

        assert_eq!(6289138340682858684, h.hash());
    }

    #[test]
    /// test that different entry content returns different hashes
    fn hash_entry_content() {
        let chain = MemChain::new();
        let t = "fooType";

        // different entries must return different hashes
        let e1 = Entry::new(t, "");
        let h1 = Header::new(&chain, &e1);

        let e2 = Entry::new(t, "a");
        let h2 = Header::new(&chain, &e2);

        assert_ne!(h1.hash(), h2.hash());

        // same entry must return same hash
        let e3 = Entry::new(t, "");
        let h3 = Header::new(&chain, &e3);

        assert_eq!(h1.hash(), h3.hash());
    }

    #[test]
    /// test that different entry types returns different hashes
    fn hash_entry_type() {
        let chain = MemChain::new();
        let t1 = "foo";
        let t2 = "bar";
        let c = "baz";

        let e1 = Entry::new(t1, c);
        let e2 = Entry::new(t2, c);

        let h1 = Header::new(&chain, &e1);
        let h2 = Header::new(&chain, &e2);

        // different types must give different hashes
        assert_ne!(h1.hash(), h2.hash());
    }

    #[test]
    /// test that different chain state returns different hashes
    fn hash_chain_state() {
        // different chain, different hash
        let mut chain = MemChain::new();
        let t = "foo";
        let c = "bar";
        let e = Entry::new(t, c);
        let h = Header::new(&chain, &e);

        let p1 = chain.push(&e);
        // p2 will have a different hash to p1 with the same entry as the chain state is different
        let p2 = chain.push(&e);

        assert_eq!(h.hash(), p1.header().hash());
        assert_ne!(h.hash(), p2.header().hash());
    }

    #[test]
    /// test that different type_next returns different hashes
    fn hash_type_next() {
        // @TODO is it possible to test that type_next changes the hash in an isolated way?
        // @see https://github.com/holochain/holochain-rust/issues/76
    }

    #[test]
    /// tests for header.validate()
    fn validate() {
        let chain = MemChain::new();
        let t = "foo";

        let e = Entry::new(t, "");
        let h = Header::new(&chain, &e);

        assert!(h.validate());
    }
}
