"""Extract and clean text from Wikipedia XML dumps."""

import bz2
import logging
import re
from pathlib import Path
from typing import Iterator, Tuple, Optional
import xml.etree.ElementTree as ET

logger = logging.getLogger(__name__)

# Regex patterns for cleaning
WIKI_PATTERNS = {
    'redirect': re.compile(r'#REDIRECT\s*\[\[.*?\]\]', re.IGNORECASE),
    'comments': re.compile(r'<!--.*?-->', re.DOTALL),
    'ref_tags': re.compile(r'<ref[^>]*>.*?</ref>', re.DOTALL),
    'html_tags': re.compile(r'<[^>]+>'),
    'wiki_links': re.compile(r'\[\[(?:[^|\]]*\|)?([^\]]+)\]\]'),
    'external_links': re.compile(r'\[http[^\s\]]+\s*([^\]]*)\]'),
    'templates': re.compile(r'\{\{[^}]+\}\}'),
    'tables': re.compile(r'\{\|.*?\|\}', re.DOTALL),
    'headers': re.compile(r'^=+\s*([^=]+)\s*=+$', re.MULTILINE),
    'lists': re.compile(r'^\*+\s*', re.MULTILINE),
    'bold_italic': re.compile(r"'''?"),
    'multiple_spaces': re.compile(r'\s+'),
    'multiple_newlines': re.compile(r'\n\n+'),
}


class WikipediaExtractor:
    """Extract clean text from Wikipedia XML dumps."""
    
    def __init__(self, dump_path: Path, language: str = 'en'):
        """Initialize the extractor.
        
        Args:
            dump_path: Path to Wikipedia dump file
            language: Language code for language-specific processing
        """
        self.dump_path = dump_path
        self.language = language
        self.namespace = {'': 'http://www.mediawiki.org/xml/export-0.10/'}
        
    def extract_articles(self, max_articles: Optional[int] = None) -> Iterator[Tuple[str, str]]:
        """Extract articles from the dump.
        
        Args:
            max_articles: Maximum number of articles to extract
            
        Yields:
            Tuple of (title, cleaned_text)
        """
        count = 0
        
        # Open the dump file (handle .bz2 compression)
        if self.dump_path.suffix == '.bz2':
            file_obj = bz2.open(self.dump_path, 'rt', encoding='utf-8')
        else:
            file_obj = open(self.dump_path, 'r', encoding='utf-8')
            
        try:
            # Use iterative parsing to handle large files
            for event, elem in ET.iterparse(file_obj, events=('start', 'end')):
                if event == 'end' and elem.tag.endswith('page'):
                    # Extract page data
                    page_data = self._extract_page(elem)
                    
                    if page_data and not self._is_redirect(page_data['text']):
                        title = page_data['title']
                        cleaned_text = self._clean_wiki_text(page_data['text'])
                        
                        if cleaned_text and len(cleaned_text) > 100:  # Skip very short articles
                            yield (title, cleaned_text)
                            count += 1
                            
                            if max_articles and count >= max_articles:
                                break
                    
                    # Clear the element to save memory
                    elem.clear()
                    
        finally:
            file_obj.close()
            
    def extract_sample(self, target_size_mb: int = 500) -> Iterator[Tuple[str, str]]:
        """Extract articles until reaching target size.
        
        Args:
            target_size_mb: Target size in megabytes
            
        Yields:
            Tuple of (title, cleaned_text)
        """
        target_size_bytes = target_size_mb * 1024 * 1024
        current_size = 0
        
        for title, text in self.extract_articles():
            text_size = len(text.encode('utf-8'))
            
            if current_size + text_size > target_size_bytes:
                # Include partial article to reach exact size
                remaining_size = target_size_bytes - current_size
                if remaining_size > 1000:  # At least 1KB
                    partial_text = text[:remaining_size]
                    yield (title, partial_text)
                break
                
            yield (title, text)
            current_size += text_size
            
    def _extract_page(self, page_elem) -> Optional[dict]:
        """Extract data from a page element."""
        data = {}
        
        for child in page_elem:
            if child.tag.endswith('title'):
                data['title'] = child.text
            elif child.tag.endswith('revision'):
                for rev_child in child:
                    if rev_child.tag.endswith('text'):
                        data['text'] = rev_child.text or ''
                        
        return data if 'title' in data and 'text' in data else None
        
    def _is_redirect(self, text: str) -> bool:
        """Check if the page is a redirect."""
        return bool(WIKI_PATTERNS['redirect'].match(text))
        
    def _clean_wiki_text(self, text: str) -> str:
        """Clean Wikipedia markup from text."""
        if not text:
            return ''
            
        # Remove redirect pages
        if self._is_redirect(text):
            return ''
            
        # Apply cleaning patterns in order
        cleaned = text
        
        # Remove comments and refs first
        cleaned = WIKI_PATTERNS['comments'].sub('', cleaned)
        cleaned = WIKI_PATTERNS['ref_tags'].sub('', cleaned)
        
        # Remove tables and templates
        cleaned = WIKI_PATTERNS['tables'].sub('', cleaned)
        cleaned = WIKI_PATTERNS['templates'].sub('', cleaned)
        
        # Convert wiki links to plain text
        cleaned = WIKI_PATTERNS['wiki_links'].sub(r'\1', cleaned)
        cleaned = WIKI_PATTERNS['external_links'].sub(r'\1', cleaned)
        
        # Remove remaining HTML tags
        cleaned = WIKI_PATTERNS['html_tags'].sub('', cleaned)
        
        # Clean headers (keep the text)
        cleaned = WIKI_PATTERNS['headers'].sub(r'\1', cleaned)
        
        # Remove list markers
        cleaned = WIKI_PATTERNS['lists'].sub('', cleaned)
        
        # Remove bold/italic markers
        cleaned = WIKI_PATTERNS['bold_italic'].sub('', cleaned)
        
        # Language-specific cleaning
        if self.language == 'ja':
            cleaned = self._clean_japanese_text(cleaned)
            
        # Normalize whitespace
        cleaned = WIKI_PATTERNS['multiple_spaces'].sub(' ', cleaned)
        cleaned = WIKI_PATTERNS['multiple_newlines'].sub('\n\n', cleaned)
        
        return cleaned.strip()
        
    def _clean_japanese_text(self, text: str) -> str:
        """Apply Japanese-specific text cleaning."""
        # Remove furigana/ruby annotations if present
        text = re.sub(r'\{\{(?:ruby|ふりがな)\|([^|]+)\|[^}]+\}\}', r'\1', text)
        
        # Normalize Japanese punctuation
        text = text.replace('．', '。')
        text = text.replace('，', '、')
        
        return text