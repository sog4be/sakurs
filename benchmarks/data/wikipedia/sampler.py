"""Generate fixed-size samples from Wikipedia dumps."""

import logging
import random
from pathlib import Path
from typing import List, Optional
from tqdm import tqdm

from .extractor import WikipediaExtractor

logger = logging.getLogger(__name__)


class WikipediaSampler:
    """Generate representative samples from Wikipedia dumps."""
    
    def __init__(self, dump_path: Path, language: str = 'en'):
        """Initialize the sampler.
        
        Args:
            dump_path: Path to Wikipedia dump file
            language: Language code
        """
        self.dump_path = dump_path
        self.language = language
        self.extractor = WikipediaExtractor(dump_path, language)
        
    def create_sample(self, output_path: Path, size_mb: int = 500, 
                     seed: Optional[int] = None) -> Path:
        """Create a fixed-size sample of Wikipedia articles.
        
        Args:
            output_path: Path to save the sample
            size_mb: Target size in megabytes
            seed: Random seed for reproducibility
            
        Returns:
            Path to the created sample
        """
        if seed is not None:
            random.seed(seed)
            
        logger.info(f"Creating {size_mb}MB sample from {self.language} Wikipedia")
        
        # Extract articles up to target size
        articles = []
        current_size = 0
        target_size = size_mb * 1024 * 1024
        
        with tqdm(total=target_size, unit='B', unit_scale=True, 
                  desc=f"Extracting {self.language} articles") as pbar:
            for title, text in self.extractor.extract_sample(size_mb):
                articles.append((title, text))
                text_size = len(text.encode('utf-8'))
                current_size += text_size
                pbar.update(text_size)
                
        logger.info(f"Extracted {len(articles)} articles")
        
        # Write to output file
        self._write_sample(articles, output_path)
        
        # Verify size
        actual_size = output_path.stat().st_size
        actual_size_mb = actual_size / (1024 * 1024)
        logger.info(f"Created sample: {actual_size_mb:.1f}MB at {output_path}")
        
        return output_path
        
    def create_stratified_sample(self, output_path: Path, size_mb: int = 500,
                               categories: Optional[List[str]] = None) -> Path:
        """Create a stratified sample with diverse content types.
        
        Args:
            output_path: Path to save the sample
            size_mb: Target size in megabytes
            categories: List of category patterns to ensure diversity
            
        Returns:
            Path to the created sample
        """
        # For now, use simple sampling
        # TODO: Implement category-based stratification
        return self.create_sample(output_path, size_mb)
        
    def _write_sample(self, articles: List[Tuple[str, str]], output_path: Path):
        """Write articles to output file."""
        output_path.parent.mkdir(parents=True, exist_ok=True)
        
        with open(output_path, 'w', encoding='utf-8') as f:
            for i, (title, text) in enumerate(articles):
                # Write article with clear boundaries
                f.write(f"===== Article {i+1}: {title} =====\n\n")
                f.write(text)
                f.write("\n\n")
                
        logger.info(f"Wrote {len(articles)} articles to {output_path}")
        
    def create_benchmark_samples(self, output_dir: Path, sizes_mb: List[int] = None):
        """Create multiple samples of different sizes for benchmarking.
        
        Args:
            output_dir: Directory to save samples
            sizes_mb: List of sample sizes in MB
        """
        if sizes_mb is None:
            sizes_mb = [10, 50, 100, 500]  # Default sizes
            
        output_dir.mkdir(parents=True, exist_ok=True)
        
        for size_mb in sizes_mb:
            output_path = output_dir / f"wikipedia_{self.language}_{size_mb}mb.txt"
            
            if output_path.exists():
                logger.info(f"Sample already exists: {output_path}")
                continue
                
            try:
                self.create_sample(output_path, size_mb, seed=42)  # Fixed seed for reproducibility
            except Exception as e:
                logger.error(f"Failed to create {size_mb}MB sample: {e}")
                continue