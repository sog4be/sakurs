"""Detailed benchmark to measure actual performance"""

import time
import sakurs
import numpy as np
from tabulate import tabulate

def measure_processing_time(text, language="en", iterations=10):
    """Measure processing time with warm-up"""
    # Create processor once
    processor = sakurs.load(language)
    
    # Warm up
    for _ in range(5):
        processor.split(text)
    
    # Measure
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        sentences = processor.split(text)
        end = time.perf_counter()
        times.append((end - start) * 1000)  # Convert to ms
    
    return {
        'mean': np.mean(times),
        'std': np.std(times),
        'min': np.min(times),
        'max': np.max(times),
        'sentences': len(sentences)
    }

def main():
    # Base text with abbreviations
    base_text = "Dr. Smith works at Apple Inc. and lives on Main St. in New York. He earned his Ph.D. from M.I.T. University. The U.S.A. is a large country. "
    
    # Test different sizes
    sizes = [1, 10, 50, 100, 200, 500, 1000, 2000, 5000]
    results = []
    
    print("Measuring performance with abbreviation-heavy text...")
    print("=" * 80)
    
    for size in sizes:
        text = base_text * size
        char_count = len(text)
        
        # Measure time
        stats = measure_processing_time(text, iterations=20 if size < 1000 else 5)
        
        # Calculate scaling factor
        if len(results) > 0:
            scaling = stats['mean'] / results[0]['time_ms']
            expected_scaling = char_count / results[0]['chars']
            efficiency = expected_scaling / scaling if scaling > 0 else 0
        else:
            scaling = 1.0
            expected_scaling = 1.0
            efficiency = 1.0
        
        results.append({
            'size': size,
            'chars': char_count,
            'sentences': stats['sentences'],
            'time_ms': stats['mean'],
            'std_ms': stats['std'],
            'scaling': scaling,
            'expected': expected_scaling,
            'efficiency': efficiency
        })
        
        print(f"Size {size:4d}: {stats['mean']:7.3f} ms (±{stats['std']:.3f})")
    
    # Display detailed table
    print("\nDetailed Performance Analysis")
    print("=" * 100)
    
    headers = ['Size', 'Characters', 'Sentences', 'Time (ms)', 'Std Dev', 'Scaling', 'Expected', 'Efficiency']
    table_data = []
    
    for r in results:
        table_data.append([
            r['size'],
            f"{r['chars']:,}",
            r['sentences'],
            f"{r['time_ms']:.3f}",
            f"±{r['std_ms']:.3f}",
            f"{r['scaling']:.2f}x",
            f"{r['expected']:.2f}x",
            f"{r['efficiency']:.1%}"
        ])
    
    print(tabulate(table_data, headers=headers, tablefmt='grid'))
    
    # Analyze scaling behavior
    print("\nScaling Analysis:")
    print("-" * 40)
    
    # Compare small to large
    small = results[0]  # size 1
    large = results[-1]  # size 5000
    
    size_increase = large['size'] / small['size']
    time_increase = large['time_ms'] / small['time_ms']
    
    print(f"Size increase: {size_increase:.0f}x")
    print(f"Time increase: {time_increase:.1f}x")
    print(f"Scaling factor: {time_increase / size_increase:.3f}")
    
    if time_increase / size_increase > 1.5:
        print("WARNING: Performance is worse than O(N) - possible O(N²) behavior!")
    elif time_increase / size_increase > 1.1:
        print("Performance is approximately O(N log N)")
    else:
        print("Performance is approximately O(N) - linear scaling")
    
    # Test with minimal abbreviations
    print("\n" + "=" * 80)
    print("Testing with minimal abbreviations...")
    
    simple_text = "This is a simple sentence. " * 100
    abbr_text = base_text * 100
    
    simple_stats = measure_processing_time(simple_text)
    abbr_stats = measure_processing_time(abbr_text)
    
    print(f"Simple text (no abbreviations): {simple_stats['mean']:.3f} ms")
    print(f"Complex text (many abbreviations): {abbr_stats['mean']:.3f} ms")
    print(f"Slowdown with abbreviations: {abbr_stats['mean'] / simple_stats['mean']:.2f}x")

if __name__ == "__main__":
    main()