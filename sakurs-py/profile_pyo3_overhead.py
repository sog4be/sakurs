"""Profile PyO3 overhead and identify bottlenecks"""

import time
import sakurs

def benchmark_sentence_detection(text, iterations=100):
    """Benchmark sentence detection with timing breakdown"""
    
    # Warm up
    for _ in range(5):
        sakurs.split(text, language="en")
    
    # Time full process
    start = time.perf_counter()
    for _ in range(iterations):
        sentences = sakurs.split(text, language="en")
    full_time = time.perf_counter() - start
    
    # Time just the processor creation (if possible)
    start = time.perf_counter()
    for _ in range(iterations):
        processor = sakurs.load("en")
    load_time = time.perf_counter() - start
    
    # Time processing with pre-loaded processor
    processor = sakurs.load("en")
    start = time.perf_counter()
    for _ in range(iterations):
        result = processor.split(text)
    process_time = time.perf_counter() - start
    
    return {
        'full_time': full_time / iterations * 1000,  # ms
        'load_time': load_time / iterations * 1000,
        'process_time': process_time / iterations * 1000,
        'sentences': len(sentences),
        'chars': len(text),
    }

def main():
    # Test with different text sizes
    base_text = "Dr. Smith works at Apple Inc. and lives on Main St. in New York. "
    
    print("PyO3 Overhead Analysis")
    print("=" * 70)
    print(f"{'Text Size':>10} | {'Full (ms)':>10} | {'Load (ms)':>10} | {'Process (ms)':>12} | {'Overhead %':>10}")
    print("-" * 70)
    
    for multiplier in [1, 10, 100, 500, 1000]:
        text = base_text * multiplier
        results = benchmark_sentence_detection(text, iterations=100 if multiplier < 100 else 10)
        
        overhead_pct = ((results['full_time'] - results['process_time']) / results['full_time']) * 100
        
        print(f"{len(text):>10} | {results['full_time']:>10.2f} | {results['load_time']:>10.2f} | {results['process_time']:>12.2f} | {overhead_pct:>9.1f}%")
    
    print("\nBreakdown for 1KB text:")
    text_1kb = base_text * 15  # ~1KB
    results = benchmark_sentence_detection(text_1kb, iterations=1000)
    
    print(f"\nTotal time: {results['full_time']:.3f} ms")
    print(f"  - Processor loading: {results['load_time']:.3f} ms ({results['load_time']/results['full_time']*100:.1f}%)")
    print(f"  - Text processing: {results['process_time']:.3f} ms ({results['process_time']/results['full_time']*100:.1f}%)")
    print(f"  - PyO3/other overhead: {results['full_time'] - results['process_time']:.3f} ms ({(results['full_time'] - results['process_time'])/results['full_time']*100:.1f}%)")

if __name__ == "__main__":
    main()