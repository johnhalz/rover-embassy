# MCAP File Indexing Guide

## Understanding MCAP File Structure

According to the [MCAP specification](https://mcap.dev/spec#summary-section), a properly indexed MCAP file has this structure:

```
[Magic]
[Header]
[Data Section with Chunks]
[Summary Section]       ← Contains indices for fast lookup
[Summary Offset Section] ← Points to summary records
[Footer]
[Magic]
```

The Summary Section contains:
- **Schema** records - Definitions of message types
- **Channel** records - Topic and encoding information
- **Chunk Index** records - Location and time range of each chunk
- **Statistics** record - Message counts and time ranges
- **Summary Offset** records - Pointers for fast random access

Without these sections, Foxglove shows: **"This file is unindexed, unindexed files may have degraded performance"**

## Why Files Become Unindexed

The Summary and Index sections are only written when `writer.finish()` is called. This happens during graceful shutdown.

### ✅ Properly Indexed File (3-5 KB)
- User presses 'q' to quit
- Shutdown signal propagates to all modules
- Logger receives shutdown signal
- `writer.finish()` writes summary section
- File is complete with index

### ❌ Unindexed File (92 bytes)
- Process killed with Ctrl+C or `kill` command
- `writer.finish()` never called
- Only header and footer written
- No summary section, no indices
- Messages may be incomplete

## How to Create Properly Indexed Files

### Method 1: Press 'q' to Quit (Recommended)

```bash
cargo run --release

# Wait for logs to accumulate
# Press 'q' when ready to quit

# You should see:
# [Logger] Finalizing MCAP file with N messages...
# [Logger] MCAP file finalized successfully (indexed)
```

### Method 2: Use the Test Example

```bash
cargo run --example test_mcap

# This creates test_indexed.mcap which is guaranteed to be indexed
# Open it in Foxglove to verify proper indexing works
```

## Verifying Your MCAP File

### File Size Check
```bash
ls -lh rover_logs_*.mcap

# Good (indexed): 2-10 KB depending on number of messages
# Bad (unindexed): 92 bytes
```

### Foxglove Check
1. Open Foxglove Studio
2. Load your `rover_logs_*.mcap` file
3. Look for warnings at the top

**No warning** = Properly indexed ✅
**"This file is unindexed"** = finish() was not called ❌

## Technical Details

### What finish() Does

When `writer.finish()` is called, the MCAP writer:

1. **Flushes any buffered data** - Writes incomplete chunks
2. **Writes Data End record** - Marks end of data section
3. **Writes Summary Section**:
   - Duplicates all Channel records
   - Duplicates all Schema records
   - Writes Chunk Index records (location and time range of each chunk)
   - Writes Statistics record (message counts, time ranges)
4. **Writes Summary Offset Section** - Pointers to summary record groups
5. **Writes Footer** - Contains offsets to summary sections
6. **Finalizes the file** - Closes the writer

### Chunking and Indexing

The MCAP Writer (v0.24) automatically:
- Batches messages into chunks (default ~1MB uncompressed)
- Compresses chunks with Zstd
- Creates message indices within each chunk
- Writes chunk indices to the summary section

This enables:
- **Fast random access** - Jump to specific timestamps
- **Efficient compression** - Better ratios with larger chunks
- **Topic filtering** - Read only specific channels
- **Time-based queries** - Find messages by log_time

## Troubleshooting

### Q: I pressed 'q' but still see "unindexed"
**A:** Check that you saw the finalization messages:
```
[Logger] Finalizing MCAP file with N messages...
[Logger] MCAP file finalized successfully (indexed)
```
If you didn't see these, the logger may not have received the shutdown signal in time.

### Q: Can I use Ctrl+C to quit?
**A:** No! Ctrl+C sends SIGINT which terminates the process immediately. The Drop implementation might help in some cases, but it's not guaranteed. Always use 'q'.

### Q: The file is 92 bytes, what happened?
**A:** The process was terminated before `writer.finish()` could run. The file only contains the header and footer. No messages or indices were written.

### Q: Can I fix an unindexed file?
**A:** No. Once the process terminates without calling finish(), the file cannot be repaired. You need to run the system again and quit with 'q'.

### Q: Does the Drop implementation help?
**A:** It helps for clean terminations (normal exit, panic), but not for signals like SIGKILL or forced termination. It's a safety net, not a guarantee.

## Best Practices

1. **Always press 'q' to quit** - Never use Ctrl+C or kill commands
2. **Wait for confirmation** - Look for "MCAP file finalized successfully"
3. **Check file size** - Indexed files are 3-10 KB minimum
4. **Test with example** - Run `cargo run --example test_mcap` to verify indexing works
5. **Keep process running** - Don't kill or interrupt during shutdown sequence

## Related Documentation

- [MCAP Specification](https://mcap.dev/spec)
- [MCAP Summary Section](https://mcap.dev/spec#summary-section)
- [Understanding MCAP Chunk Size and Compression](https://foxglove.dev/blog/understanding-mcap-chunk-size-and-compression)
- [Foxglove Studio](https://foxglove.dev/download)
