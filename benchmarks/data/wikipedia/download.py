#!/usr/bin/env python3
"""Download Wikipedia dumps for benchmarking."""

import os
import sys
import logging
from pathlib import Path
from datetime import datetime
from typing import Optional, Dict
import click
import requests
from tqdm import tqdm

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

# Wikipedia dump configurations
WIKIPEDIA_DUMPS = {
    'en': {
        'name': 'English Wikipedia',
        'base_url': 'https://dumps.wikimedia.org/enwiki',
        'file_pattern': 'enwiki-{date}-pages-articles-multistream.xml.bz2'
    },
    'ja': {
        'name': 'Japanese Wikipedia',
        'base_url': 'https://dumps.wikimedia.org/jawiki',
        'file_pattern': 'jawiki-{date}-pages-articles-multistream.xml.bz2'
    }
}

# Cache directory
CACHE_DIR = Path(__file__).parent / "cache"


def get_latest_dump_date(language: str) -> Optional[str]:
    """Get the latest available dump date for a language.
    
    Args:
        language: Language code
        
    Returns:
        Date string (YYYYMMDD) or None
    """
    config = WIKIPEDIA_DUMPS.get(language)
    if not config:
        raise ValueError(f"Unsupported language: {language}")
        
    # Check the dumps index
    index_url = f"{config['base_url']}/latest/"
    
    try:
        response = requests.head(index_url, timeout=10)
        if response.status_code == 200:
            return "latest"
        else:
            # Try recent dates
            from datetime import timedelta
            today = datetime.now()
            
            for days_back in range(0, 60, 20):  # Check every 20 days
                date = today - timedelta(days=days_back)
                date_str = date.strftime("%Y%m%d")
                test_url = f"{config['base_url']}/{date_str}/"
                
                response = requests.head(test_url, timeout=5)
                if response.status_code == 200:
                    return date_str
                    
    except Exception as e:
        logger.warning(f"Failed to check dump dates: {e}")
        
    return None


def download_wikipedia_dump(language: str, date: str, output_dir: Path) -> Path:
    """Download Wikipedia dump file.
    
    Args:
        language: Language code
        date: Dump date or "latest"
        output_dir: Output directory
        
    Returns:
        Path to downloaded file
    """
    config = WIKIPEDIA_DUMPS.get(language)
    if not config:
        raise ValueError(f"Unsupported language: {language}")
        
    filename = config['file_pattern'].format(date=date)
    if date == "latest":
        url = f"{config['base_url']}/latest/{filename}"
    else:
        url = f"{config['base_url']}/{date}/{filename}"
        
    output_path = output_dir / filename
    
    # Check if already downloaded
    if output_path.exists():
        logger.info(f"Dump already downloaded: {output_path}")
        return output_path
        
    logger.info(f"Downloading {config['name']} dump from {url}")
    logger.info("This may take a while (file size: several GB)")
    
    try:
        response = requests.get(url, stream=True, timeout=30)
        response.raise_for_status()
        
        total_size = int(response.headers.get('content-length', 0))
        
        with open(output_path, 'wb') as f:
            with tqdm(total=total_size, unit='B', unit_scale=True, desc=f"Downloading {language}") as pbar:
                for chunk in response.iter_content(chunk_size=8192):
                    f.write(chunk)
                    pbar.update(len(chunk))
                    
        logger.info(f"Downloaded successfully: {output_path}")
        return output_path
        
    except Exception as e:
        # Clean up partial download
        if output_path.exists():
            output_path.unlink()
        raise RuntimeError(f"Download failed: {e}")


def get_dump_info(language: str) -> Dict[str, str]:
    """Get information about available dumps.
    
    Args:
        language: Language code
        
    Returns:
        Dictionary with dump information
    """
    config = WIKIPEDIA_DUMPS.get(language, {})
    latest_date = get_latest_dump_date(language)
    
    return {
        'language': language,
        'name': config.get('name', 'Unknown'),
        'latest_date': latest_date or 'Unknown',
        'base_url': config.get('base_url', ''),
        'index_url': f"{config.get('base_url', '')}/backup-index.html"
    }


@click.command()
@click.option('--language', '-l', type=click.Choice(['en', 'ja']), required=True,
              help='Language to download')
@click.option('--date', '-d', default='latest',
              help='Dump date (YYYYMMDD) or "latest"')
@click.option('--output-dir', '-o', type=click.Path(),
              default=str(CACHE_DIR / 'dumps'),
              help='Output directory for dumps')
@click.option('--info-only', is_flag=True,
              help='Show dump information without downloading')
def main(language, date, output_dir, info_only):
    """Download Wikipedia dumps for benchmarking."""
    if info_only:
        info = get_dump_info(language)
        print(f"\nWikipedia Dump Information for {info['name']}:")
        print(f"  Latest available: {info['latest_date']}")
        print(f"  Index URL: {info['index_url']}")
        print(f"  Base URL: {info['base_url']}")
        return
        
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Get the latest date if needed
    if date == 'latest':
        actual_date = get_latest_dump_date(language)
        if not actual_date:
            logger.error(f"Could not determine latest dump date for {language}")
            sys.exit(1)
        logger.info(f"Using latest dump date: {actual_date}")
    else:
        actual_date = date
        
    try:
        dump_path = download_wikipedia_dump(language, actual_date, output_dir)
        print(f"\nDump downloaded to: {dump_path}")
        
        # Show file size
        size_mb = dump_path.stat().st_size / (1024 * 1024)
        print(f"File size: {size_mb:.1f} MB")
        
    except Exception as e:
        logger.error(f"Failed to download dump: {e}")
        sys.exit(1)


if __name__ == '__main__':
    main()