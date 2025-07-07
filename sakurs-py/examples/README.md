# Sakurs Python Examples

This directory contains examples demonstrating how to use the sakurs Python package for sentence boundary detection.

## Examples

### 1. Basic Usage (`basic_usage.py`)
- Simple sentence splitting
- Working with different languages
- Using the Processor class for multiple texts
- Handling edge cases

### 2. Performance Tuning (`performance_tuning.py`)
- Configuring chunk sizes
- Thread configuration
- Benchmarking different settings
- Tips for optimal performance

### 3. Error Handling (`error_handling.py`)
- Handling unsupported languages
- Dealing with edge case inputs
- Implementing fallback strategies
- Working with deprecated features

## Running the Examples

First, make sure sakurs is installed:

```bash
pip install sakurs
```

Then run any example:

```bash
python basic_usage.py
python performance_tuning.py
python error_handling.py
```

## Key Takeaways

1. **Simple API**: Most users only need `sakurs.split(text)`
2. **Language Support**: Currently supports English and Japanese
3. **Performance**: Reuse Processor instances for better performance
4. **Configuration**: Tune chunk_size and num_threads based on your use case
5. **Error Handling**: The library handles edge cases gracefully