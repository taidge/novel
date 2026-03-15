// ========== Theme Toggle ==========
function toggleTheme() {
    const html = document.documentElement;
    const current = html.getAttribute('data-theme');
    const next = current === 'dark' ? 'light' : 'dark';
    html.setAttribute('data-theme', next);
    localStorage.setItem('novel-theme', next);
}

// Restore saved theme
(function() {
    const saved = localStorage.getItem('novel-theme');
    if (saved) {
        document.documentElement.setAttribute('data-theme', saved);
    } else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
        document.documentElement.setAttribute('data-theme', 'dark');
    }
})();

// ========== Banner Dismiss ==========
function dismissBanner() {
    const banner = document.getElementById('site-banner');
    if (banner) {
        banner.classList.add('dismissed');
        sessionStorage.setItem('novel-banner-dismissed', '1');
    }
}
(function() {
    if (sessionStorage.getItem('novel-banner-dismissed') === '1') {
        const banner = document.getElementById('site-banner');
        if (banner) banner.classList.add('dismissed');
    }
})();

// ========== Mobile Sidebar Toggle ==========
function toggleSidebar() {
    const sidebar = document.getElementById('sidebar');
    if (sidebar) {
        sidebar.classList.toggle('open');
        document.body.classList.toggle('sidebar-open');
    }
}

// ========== TOC Active State ==========
(function() {
    const tocLinks = document.querySelectorAll('.toc-item a');
    if (tocLinks.length === 0) return;

    const headings = [];
    tocLinks.forEach(link => {
        const id = link.getAttribute('href')?.slice(1);
        if (id) {
            const el = document.getElementById(id);
            if (el) headings.push({ el, link });
        }
    });

    function updateActive() {
        let current = null;
        for (const { el } of headings) {
            if (el.getBoundingClientRect().top <= 100) {
                current = el;
            }
        }
        tocLinks.forEach(l => l.classList.remove('active'));
        if (current) {
            const active = headings.find(h => h.el === current);
            if (active) active.link.classList.add('active');
        }
    }

    window.addEventListener('scroll', updateActive, { passive: true });
    updateActive();
})();

// ========== Search with Keyboard Navigation & Highlighting ==========
(function() {
    const input = document.getElementById('search-input');
    const results = document.getElementById('search-results');
    if (!input || !results) return;

    let searchIndex = null;
    let focusedIdx = -1;

    async function loadIndex() {
        if (searchIndex) return;
        try {
            const base = document.querySelector('link[rel="stylesheet"]')?.href?.replace('assets/style.css', '') || '/';
            const resp = await fetch(base + 'assets/search-index.json');
            searchIndex = await resp.json();
        } catch(e) {
            console.warn('Failed to load search index:', e);
        }
    }

    function highlightMatch(text, query) {
        if (!query) return text;
        const idx = text.toLowerCase().indexOf(query.toLowerCase());
        if (idx < 0) return text;
        return text.slice(0, idx) + '<mark>' + text.slice(idx, idx + query.length) + '</mark>' + text.slice(idx + query.length);
    }

    function search(query) {
        if (!searchIndex || !query) return [];
        const q = query.toLowerCase();

        // Score-based search: title matches score higher
        const scored = searchIndex
            .map(item => {
                let score = 0;
                if (item.title.toLowerCase().includes(q)) score += 10;
                if (item.headers.some(h => h.toLowerCase().includes(q))) score += 5;
                if (item.content.toLowerCase().includes(q)) score += 1;
                return { ...item, score };
            })
            .filter(item => item.score > 0)
            .sort((a, b) => b.score - a.score)
            .slice(0, 8);

        return scored.map(item => {
            const idx = item.content.toLowerCase().indexOf(q);
            let preview = '';
            if (idx >= 0) {
                const start = Math.max(0, idx - 40);
                const end = Math.min(item.content.length, idx + 60);
                preview = (start > 0 ? '...' : '') + item.content.slice(start, end) + (end < item.content.length ? '...' : '');
                preview = highlightMatch(preview, query);
            }
            return { ...item, preview, highlightedTitle: highlightMatch(item.title, query) };
        });
    }

    function renderResults(items, query) {
        focusedIdx = -1;
        if (items.length === 0) {
            results.innerHTML = '<div class="search-result-item"><div class="search-result-title">No results</div></div>';
        } else {
            results.innerHTML = items.map((item, i) => `
                <a href="${item.route_path}" class="search-result-item" data-idx="${i}" style="display:block;color:inherit;text-decoration:none">
                    <div class="search-result-title">${item.highlightedTitle || item.title}</div>
                    ${item.preview ? `<div class="search-result-preview">${item.preview}</div>` : ''}
                </a>
            `).join('');
        }
        results.classList.add('active');
    }

    function setFocus(idx) {
        const items = results.querySelectorAll('.search-result-item[data-idx]');
        items.forEach(el => el.classList.remove('focused'));
        if (idx >= 0 && idx < items.length) {
            focusedIdx = idx;
            items[idx].classList.add('focused');
            items[idx].scrollIntoView({ block: 'nearest' });
        }
    }

    input.addEventListener('focus', loadIndex);

    input.addEventListener('input', () => {
        const q = input.value.trim();
        if (!q) {
            results.classList.remove('active');
            results.innerHTML = '';
            return;
        }
        const items = search(q);
        renderResults(items, q);
    });

    input.addEventListener('keydown', (e) => {
        const items = results.querySelectorAll('.search-result-item[data-idx]');
        const count = items.length;

        if (e.key === 'ArrowDown') {
            e.preventDefault();
            setFocus(focusedIdx < count - 1 ? focusedIdx + 1 : 0);
        } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            setFocus(focusedIdx > 0 ? focusedIdx - 1 : count - 1);
        } else if (e.key === 'Enter' && focusedIdx >= 0) {
            e.preventDefault();
            const focused = items[focusedIdx];
            if (focused) window.location.href = focused.getAttribute('href');
        } else if (e.key === 'Escape') {
            results.classList.remove('active');
            input.blur();
        }
    });

    // Global keyboard shortcut: / to focus search
    document.addEventListener('keydown', (e) => {
        if (e.key === '/' && document.activeElement !== input && !['INPUT', 'TEXTAREA'].includes(document.activeElement.tagName)) {
            e.preventDefault();
            input.focus();
        }
    });

    document.addEventListener('click', (e) => {
        if (!e.target.closest('.search-box')) {
            results.classList.remove('active');
        }
    });
})();

// ========== Copy Button Feedback ==========
document.addEventListener('click', (e) => {
    const btn = e.target.closest('.copy-btn');
    if (btn) {
        const orig = btn.textContent;
        btn.textContent = 'Copied!';
        setTimeout(() => { btn.textContent = orig; }, 1500);
    }
});

// ========== Tabs ==========
document.addEventListener('click', (e) => {
    const btn = e.target.closest('.tab-btn');
    if (!btn) return;
    const container = btn.closest('.tabs-container');
    if (!container) return;

    const tabId = btn.dataset.tab;

    // Update buttons
    container.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');

    // Update panels
    container.querySelectorAll('.tab-panel').forEach(p => {
        p.classList.toggle('active', p.dataset.tab === tabId);
    });
});

// ========== Image Zoom ==========
document.addEventListener('click', (e) => {
    const img = e.target.closest('img.zoomable');
    if (!img) return;

    const overlay = document.createElement('div');
    overlay.className = 'image-zoom-overlay';
    const zoomed = document.createElement('img');
    zoomed.src = img.src;
    zoomed.alt = img.alt;
    overlay.appendChild(zoomed);
    document.body.appendChild(overlay);

    overlay.addEventListener('click', () => overlay.remove());
    document.addEventListener('keydown', function handler(e) {
        if (e.key === 'Escape') {
            overlay.remove();
            document.removeEventListener('keydown', handler);
        }
    });
});

// ========== Back to Top ==========
(function() {
    const btn = document.getElementById('back-to-top');
    if (!btn) return;

    window.addEventListener('scroll', () => {
        btn.classList.toggle('visible', window.scrollY > 300);
    }, { passive: true });
})();
