"""Unified loader interface for Wikipedia data."""

import json
import logging
from pathlib import Path
from typing import Dict, List, Tuple, Iterator, Optional
import sys

# Add parent directory for base_loader import
sys.path.insert(0, str(Path(__file__).parent.parent))
from base_loader import CorpusLoader

from .download import download_wikipedia_dump, get_latest_dump_date
from .extractor import WikipediaExtractor
from .sampler import WikipediaSampler

logger = logging.getLogger(__name__)


class WikipediaLoader(CorpusLoader):
    """Loader for Wikipedia performance benchmarking data."""
    
    def __init__(self, language: str = 'en', size_mb: int = 500, 
                 cache_dir: Optional[Path] = None):
        """Initialize the Wikipedia loader.
        
        Args:
            language: Language code ('en' or 'ja')
            size_mb: Target sample size in MB
            cache_dir: Cache directory
        """
        super().__init__(cache_dir)
        self.language = language
        self.size_mb = size_mb
        self.sample_file = self.cache_dir / f"wikipedia_{language}_{size_mb}mb.txt"
        self.dump_dir = self.cache_dir / "dumps"
        
    def is_cached(self) -> bool:
        """Check if sample is already cached."""
        return self.sample_file.exists()
        
    def download(self, force: bool = False) -> Path:
        """Download and prepare Wikipedia sample.
        
        Args:
            force: Force re-download even if cached
            
        Returns:
            Path to prepared sample
        """
        if self.is_cached() and not force:
            logger.info(f"Using cached sample: {self.sample_file}")
            return self.sample_file
            
        # Download the dump first
        self.dump_dir.mkdir(parents=True, exist_ok=True)
        
        # Get latest dump date
        dump_date = get_latest_dump_date(self.language)
        if not dump_date:
            raise RuntimeError(f"Could not determine dump date for {self.language}")
            
        logger.info(f"Downloading {self.language} Wikipedia dump (date: {dump_date})")
        
        try:
            dump_path = download_wikipedia_dump(self.language, dump_date, self.dump_dir)
        except Exception as e:
            raise RuntimeError(f"Failed to download Wikipedia dump: {e}")
            
        # Extract sample
        logger.info(f"Extracting {self.size_mb}MB sample from dump")
        sampler = WikipediaSampler(dump_path, self.language)
        sample_path = sampler.create_sample(self.sample_file, self.size_mb, seed=42)
        
        return sample_path
        
    def load(self) -> Dict:
        """Load the Wikipedia sample.
        
        Returns:
            Dictionary with sample data and metadata
        """
        if not self.is_cached():
            logger.info("Sample not found, downloading...")
            self.download()
            
        # Read the sample file
        with open(self.sample_file, 'r', encoding='utf-8') as f:
            content = f.read()
            
        # Split into articles
        articles = []
        current_article = {'title': '', 'text': ''}
        
        for line in content.split('\n'):
            if line.startswith('===== Article'):
                if current_article['text']:
                    articles.append(current_article)
                # Extract title from header
                title = line.split(':', 1)[1].strip().rstrip(' =====') if ':' in line else 'Unknown'
                current_article = {'title': title, 'text': ''}
            else:
                current_article['text'] += line + '\n'
                
        if current_article['text']:
            articles.append(current_article)
            
        return {
            'metadata': {
                'corpus': f'Wikipedia-{self.language.upper()}',
                'language': self.language,
                'size_mb': self.size_mb,
                'articles': len(articles)
            },
            'articles': articles
        }
        
    def iter_documents(self) -> Iterator[Tuple[str, List[str]]]:
        """Iterate over articles.
        
        Note: Wikipedia samples don't have sentence boundaries,
        so we return each article as a single "sentence".
        
        Yields:
            Tuple of (article_text, [article_text])
        """
        data = self.load()
        
        for article in data['articles']:
            text = article['text'].strip()
            if text:
                # Return article as both document and single sentence
                # Real sentence segmentation will be done by the benchmarked tools
                yield text, [text]
                
    def to_plain_text(self, output_path: Path) -> Path:
        """Copy sample to output path.
        
        Args:
            output_path: Path to save plain text
            
        Returns:
            Path to output file
        """
        if not self.is_cached():
            self.download()
            
        # Copy the sample file
        import shutil
        shutil.copy2(self.sample_file, output_path)
        
        return output_path
        
    def to_sentences_file(self, output_path: Path) -> Path:
        """Convert to one-article-per-line format.
        
        Args:
            output_path: Path to save sentences
            
        Returns:
            Path to output file
        """
        data = self.load()
        
        with open(output_path, 'w', encoding='utf-8') as f:
            for article in data['articles']:
                text = article['text'].strip()
                if text:
                    # Replace newlines with spaces for one-line format
                    one_line = ' '.join(text.split())
                    f.write(one_line + '\n')
                    
        return output_path
        
    def get_statistics(self) -> Dict[str, any]:
        """Get Wikipedia sample statistics."""
        data = self.load()
        
        total_chars = sum(len(article['text']) for article in data['articles'])
        total_words = sum(len(article['text'].split()) for article in data['articles'])
        
        stats = {
            'corpus': data['metadata']['corpus'],
            'language': self.language,
            'size_mb': self.size_mb,
            'articles': len(data['articles']),
            'total_characters': total_chars,
            'total_words': total_words,
            'avg_article_length': total_chars / len(data['articles']) if data['articles'] else 0
        }
        
        # Language-specific stats
        if self.language == 'ja':
            # Count character types for Japanese
            hiragana = katakana = kanji = 0
            for article in data['articles']:
                for char in article['text']:
                    if '\u3040' <= char <= '\u309f':
                        hiragana += 1
                    elif '\u30a0' <= char <= '\u30ff':
                        katakana += 1
                    elif '\u4e00' <= char <= '\u9fff':
                        kanji += 1
                        
            stats.update({
                'hiragana_count': hiragana,
                'katakana_count': katakana,
                'kanji_count': kanji
            })
            
        return stats