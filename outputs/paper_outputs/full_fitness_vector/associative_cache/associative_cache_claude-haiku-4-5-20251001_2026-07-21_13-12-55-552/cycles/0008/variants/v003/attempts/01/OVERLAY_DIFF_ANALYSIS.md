# Detailed Overlay Differences - v003 Variant

## File 1: src/associative_cache/mod.rs

### Struct Changes

**BEFORE (Original):**
```rust
#[derive(Debug)]
pub struct AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub buckets: Vec<AssociativeCacheBucket<K, V>>,
}
```

**AFTER (Overlay):**
```rust
#[derive(Debug)]
pub struct AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub buckets: Vec<AssociativeCacheBucket<K, V>>,
	/// Rolling buffer of hit/miss outcomes: true = hit, false = miss
	/// Used to predict workload reuse patterns for insertion positioning
	hit_miss_buffer: [bool; 16],
	/// Current write position in the hit/miss buffer (circular)
	buffer_index: usize,
}
```

**IMPACT:** +2 fields
- `hit_miss_buffer`: Fixed-size array (2 bits * 16 = 2 bytes overhead per cache instance)
- `buffer_index`: usize overhead (~8 bytes on 64-bit systems)

### Constructor Changes

**BEFORE (Original):**
```rust
pub fn try_new(settings: AssociativeCacheSettings) -> Option<Self> {
	// ... validation ...
	let mut buckets = Vec::with_capacity(settings.num_of_buckets);
	for _ in 0..settings.num_of_buckets {
		buckets.push(AssociativeCacheBucket::new(&settings));
	}
	Some(Self { buckets })
}
```

**AFTER (Overlay):**
```rust
pub fn try_new(settings: AssociativeCacheSettings) -> Option<Self> {
	// ... validation ...
	let mut buckets = Vec::with_capacity(settings.num_of_buckets);
	for _ in 0..settings.num_of_buckets {
		buckets.push(AssociativeCacheBucket::new(&settings));
	}
	Some(Self {
		buckets,
		hit_miss_buffer: [false; 16],
		buffer_index: 0,
	})
}
```

**IMPACT:** Initialization of new fields

### New Helper Methods

**ADDED:**
```rust
/// Record a hit or miss in the rolling buffer
fn record_outcome(&mut self, is_hit: bool) {
	self.hit_miss_buffer[self.buffer_index] = is_hit;
	self.buffer_index = (self.buffer_index + 1) % 16;
}

/// Sample the recent history and return true if >50% were hits
fn should_insert_at_head(&self) -> bool {
	let hits = self.hit_miss_buffer.iter().filter(|&&x| x).count();
	hits > 8 // 8 > 16/2
}
```

**IMPACT:** +2 new methods (private, no API change)

### Method Modifications

**`put()` BEFORE (Original):**
```rust
pub fn put(&mut self, key: K, value: V) {
	debug!("AssociativeCache.put: key={key:?} bucket={}", get_index(&key, self.buckets.len()));
	let bucket = self.get_mut_bucket(&key);
	bucket.put(Item::new(key, value));
}
```

**`put()` AFTER (Overlay):**
```rust
pub fn put(&mut self, key: K, value: V) {
	debug!("AssociativeCache.put: key={key:?} bucket={}", get_index(&key, self.buckets.len()));
	let bucket = self.get_mut_bucket(&key);

	// Determine insertion preference based on recent hit/miss history
	let prefer_head = self.should_insert_at_head();
	bucket.set_insertion_preference(prefer_head);

	bucket.put(Item::new(key, value));
}
```

**IMPACT:** 
- Adds workload analysis logic
- Calls new `set_insertion_preference()` on bucket
- Signature unchanged, return type unchanged

**`get_and_update_item()` BEFORE (Original):**
```rust
pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
	let bucket_index = get_index(key, self.buckets.len());
	let bucket = &mut self.buckets[bucket_index];
	let data = bucket.get(key);
	debug!("AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit={}", data.is_some());
	Some(&data?.value)
}
```

**`get_and_update_item()` AFTER (Overlay):**
```rust
pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
	let bucket_index = get_index(key, self.buckets.len());
	let bucket = &mut self.buckets[bucket_index];
	let data = bucket.get(key);

	// Record hit/miss in the buffer
	let is_hit = data.is_some();
	self.record_outcome(is_hit);

	debug!("AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit={is_hit}");
	Some(&data?.value)
}
```

**IMPACT:**
- Adds outcome recording
- Signature unchanged, return type unchanged
- Debug output improved (variable reused)

---

## File 2: src/associative_cache/associative_cache_bucket.rs

### Struct Changes

**BEFORE (Original):**
```rust
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
}
```

**AFTER (Overlay):**
```rust
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Whether to insert new items at head (true) or tail (false)
	prefer_head_insertion: bool,
}
```

**IMPACT:** +1 field
- `prefer_head_insertion`: bool overhead (1 byte, likely padded to alignment)

### Constructor Changes

**BEFORE (Original):**
```rust
pub fn new(settings: &AssociativeCacheSettings) -> Self {
	Self {
		items: Vec::with_capacity(settings.bucket_size),
		max_size: settings.bucket_size,
		eviction_policy: settings.eviction_policy.build(),
	}
}
```

**AFTER (Overlay):**
```rust
pub fn new(settings: &AssociativeCacheSettings) -> Self {
	Self {
		items: Vec::with_capacity(settings.bucket_size),
		max_size: settings.bucket_size,
		eviction_policy: settings.eviction_policy.build(),
		prefer_head_insertion: false,
	}
}
```

**IMPACT:** Initialization of new field (conservative default: tail insertion)

### New Method

**ADDED:**
```rust
/// Set the insertion preference for the next put operation
pub fn set_insertion_preference(&mut self, prefer_head: bool) {
	self.prefer_head_insertion = prefer_head;
}
```

**IMPACT:** +1 new public method (called by cache for every put)

### Trait Implementation Override

**BEFORE (Original):**
Uses default implementation from `Bucket<K, V>` trait:
```rust
fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
	if let Some(position) = self.get_position(&item.key) {
		// Update existing key - behavior unchanged
		trace!("Bucket.put: key={:?} update_existing", item.key);
		self.get_items_mut()[position].value = item.value;
		let pushed_item = self.get_from_position(position);
		return (pushed_item, None);
	}
	let evicted_item = if self.is_full() { self.evict_item() } else { None };
	self.get_items_mut().push(item);  // <-- Always pushes (tail)
	let pushed_item = self.get_items().last().expect("bucket invariant: just pushed");
	(pushed_item, evicted_item)
}
```

**AFTER (Overlay):**
Custom implementation:
```rust
fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
	if let Some(position) = self.get_position(&item.key) {
		// Update existing key - behavior unchanged
		log::trace!("Bucket.put: key={:?} update_existing", item.key);
		self.get_items_mut()[position].value = item.value;
		let pushed_item = self.get_from_position(position);
		return (pushed_item, None);
	}

	// New item insertion - decide position based on preference
	let evicted_item = if self.is_full() {
		self.evict_item()
	} else {
		None
	};

	// Insert at head (back, protected) or tail (front, eviction) based on preference
	// Head = items.push() (back/end of vector) - protected from eviction, for hot items
	// Tail = items.insert(0, ...) (front/start) - vulnerable to eviction, for cold items
	if self.prefer_head_insertion {
		// Hot workload: insert at head (back/end) for retention
		self.items.push(item);
	} else {
		// Cold workload: insert at tail (front/start) for quick eviction
		self.items.insert(0, item);
	}

	let pushed_item = if self.prefer_head_insertion {
		self.items.last().expect("bucket invariant: just pushed")
	} else {
		&self.items[0]
	};

	(pushed_item, evicted_item)
}
```

**IMPACT:**
- Overrides trait default implementation
- Changes insertion position (push vs insert(0)) based on `prefer_head_insertion`
- Updates reference retrieval to match position
- Return type unchanged: `(&Item<K, V>, Option<Item<K, V>>)`

---

## Summary of Changes

| Aspect | Before | After | Impact |
|--------|--------|-------|--------|
| AssociativeCache fields | 1 | 3 | +10-16 bytes per instance |
| AssociativeCacheBucket fields | 3 | 4 | +1-8 bytes per bucket |
| New methods (Cache) | 0 | 2 | +2 private methods |
| New methods (Bucket) | 0 | 1 | +1 public method |
| Trait implementations | 1 (default) | 1 (custom) | Override only |
| Signatures changed | 0 | 0 | Complete binary compatibility |
| Return types changed | 0 | 0 | Complete binary compatibility |

## Behavioral Changes

1. **Insertion Position:** Items can now be inserted at the end (push) or beginning (insert(0))
2. **Insertion Strategy:** Decision based on recent hit/miss history
3. **Hot vs Cold Items:** Hot workloads insert at tail (protected), cold at head (vulnerable)
4. **Eviction Logic:** Unchanged - still determined by policy at index 0 or policy's candidate

## Invariant Preservation

✅ **Capacity Invariant:** Not affected
- Eviction still happens when full
- Total items still bounded by max_size

✅ **Slot Legality:** Not affected
- Key hashing unchanged
- Keys still hash to same bucket

✅ **Eviction Policy:** Not affected
- Policy still determines eviction candidate
- Only insertion position changes

✅ **Trait Compliance:** Maintained
- `Bucket<K, V>` trait still fully implemented
- `Cache<K, V>` trait still fully implemented
