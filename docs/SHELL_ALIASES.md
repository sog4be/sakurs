# Useful Shell Aliases for Sakurs

## Basic Aliases

```bash
# Process with specific output formats
alias sakurs-json='sakurs process -f json'
alias sakurs-md='sakurs process -f markdown'

# Language-specific processing
alias sakurs-en='sakurs process -l english'
alias sakurs-ja='sakurs process -l japanese'

# Performance profiles
alias sakurs-fast='sakurs process --threads 1'  # Sequential for small files
alias sakurs-turbo='sakurs process --parallel'  # Force parallel

# Common workflows
alias sakurs-stdin='sakurs process -i -'
alias sakurs-quiet='sakurs process -q'
alias sakurs-verbose='sakurs process -vv'
```

## Advanced Aliases

```bash
# Process all text files in directory
alias sakurs-dir='sakurs process -i "*.txt"'

# Stream large files
alias sakurs-stream='sakurs process --stream'

# Japanese novel processing
alias sakurs-novel='sakurs process -l japanese -f markdown'

# Log file processing (fast mode)
alias sakurs-logs='sakurs process --threads 1 -f json'
```

## Function Examples

```bash
# Process with custom thread count based on file size
sakurs-auto() {
    local file="$1"
    local size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)
    local threads=1
    
    if [ "$size" -gt 10485760 ]; then  # 10MB
        threads=4
    elif [ "$size" -gt 1048576 ]; then  # 1MB
        threads=2
    fi
    
    sakurs process -i "$file" --threads "$threads" "${@:2}"
}

# Batch process with progress
sakurs-batch() {
    local pattern="${1:-*.txt}"
    find . -name "$pattern" -type f | while read -r file; do
        echo "Processing: $file"
        sakurs process -i "$file" -o "${file%.txt}_sentences.txt"
    done
}

# Benchmark different thread counts
sakurs-benchmark() {
    local file=$1
    echo "Testing file: $file ($(du -h "$file" | cut -f1))"
    
    for threads in 1 2 4 8; do
        echo -n "Threads: $threads - "
        time -p sakurs process -i "$file" --threads $threads -o /dev/null 2>&1 | grep real
    done
}
```

## Adding Aliases to Your Shell

To use these aliases, add them to your shell configuration file:

- For bash: `~/.bashrc` or `~/.bash_profile`
- For zsh: `~/.zshrc`
- For fish: `~/.config/fish/config.fish` (with slight syntax modifications)

After adding the aliases, reload your shell configuration:
```bash
source ~/.bashrc  # or ~/.zshrc
```

## Tips

1. **Combine aliases**: You can use aliases together
   ```bash
   sakurs-ja-json() { sakurs process -l japanese -f json "$@"; }
   ```

2. **Override defaults**: Aliases can be overridden with explicit flags
   ```bash
   sakurs-json -f text file.txt  # Will use text format, not JSON
   ```

3. **Check alias expansion**: Use `type` to see what an alias expands to
   ```bash
   type sakurs-json
   # sakurs-json is aliased to 'sakurs process -f json'
   ```