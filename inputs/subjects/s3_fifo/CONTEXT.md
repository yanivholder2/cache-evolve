# S3-FIFO Subject

This subject implements the S3-FIFO cache replacement policy from
"FIFO Queues are ALL You Need for Cache Eviction" by Juncheng Yang et al.

The policy keeps a small FIFO admission queue, a larger main FIFO resident
queue, and a ghost queue of recently evicted keys. Reaccesses to ghosted
items promote them into the main resident queue.

The JNI boundary lives in `src/s3_fifo/s3_fifo_jni.rs`. The subject is sized by
`--maximumSize` alone; the internal S3-FIFO partitioning is computed from the
configured capacity.
