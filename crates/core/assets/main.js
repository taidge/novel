// ========== Theme Toggle ==========
function toggleTheme() {
    const html = document.documentElement;
    const current = html.getAttribute('data-theme');
    const next = current === 'dark' ? 'light' : 'dark';
    html.setAttribute('data-theme', next);
    localStorage.setItem('novel-theme', next);
    // Update aria-pressed on theme toggle button
    const btn = document.querySelector('.theme-toggle');
    if (btn) btn.setAttribute('aria-pressed', next === 'dark' ? 'true' : 'false');
}

// Restore saved theme
(function() {
    const saved = localStorage.getItem('novel-theme');
    if (saved) {
        document.documentElement.setAttribute('data-theme', saved);
    } else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
        document.documentElement.setAttribute('data-theme', 'dark');
    }
    // Set initial aria-pressed
    requestAnimationFrame(function() {
        const btn = document.querySelector('.theme-toggle');
        if (btn) {
            const theme = document.documentElement.getAttribute('data-theme');
            btn.setAttribute('aria-pressed', theme === 'dark' ? 'true' : 'false');
        }
    });
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
        const isOpen = sidebar.classList.toggle('open');
        document.body.classList.toggle('sidebar-open');
        // Update aria-expanded on sidebar toggle button
        const btn = document.querySelector('.sidebar-toggle');
        if (btn) btn.setAttribute('aria-expanded', isOpen ? 'true' : 'false');
    }
}

// ========== Page Progress Indicator ==========
(function() {
    const progress = document.getElementById('doc-progress');
    if (!progress) return;

    function updateProgress() {
        const docHeight = document.documentElement.scrollHeight - window.innerHeight;
        if (docHeight <= 0) { progress.style.width = '0'; return; }
        const pct = Math.min(100, (window.scrollY / docHeight) * 100);
        progress.style.width = pct + '%';
    }

    window.addEventListener('scroll', updateProgress, { passive: true });
    updateProgress();
})();

// ========== TOC Active State ==========
(function() {
    const tocLinks = document.querySelectorAll('.toc-item a');
    if (tocLinks.length === 0) return;

    const headings = [];
    tocLinks.forEach(function(link) {
        const id = link.getAttribute('href');
        if (id) {
            const el = document.getElementById(id.slice(1));
            if (el) headings.push({ el: el, link: link });
        }
    });

    function updateActive() {
        var current = null;
        for (var i = 0; i < headings.length; i++) {
            if (headings[i].el.getBoundingClientRect().top <= 100) {
                current = headings[i].el;
            }
        }
        tocLinks.forEach(function(l) { l.classList.remove('active'); });
        if (current) {
            for (var j = 0; j < headings.length; j++) {
                if (headings[j].el === current) {
                    headings[j].link.classList.add('active');
                    break;
                }
            }
        }
    }

    window.addEventListener('scroll', updateActive, { passive: true });
    updateActive();
})();

// ========== Search with Fuzzy Matching & Section-Level Results ==========
(function() {
    var input = document.getElementById('search-input');
    var results = document.getElementById('search-results');
    if (!input || !results) return;

    var searchIndex = null;
    var focusedIdx = -1;
    var debounceTimer = null;

    function loadIndex() {
        if (searchIndex) return Promise.resolve();
        var base = '';
        var link = document.querySelector('link[rel="stylesheet"]');
        if (link && link.href) {
            var idx = link.href.indexOf('assets/');
            if (idx >= 0) base = link.href.substring(0, idx);
        }
        if (!base) base = '/';
        return fetch(base + 'assets/search-index.json')
            .then(function(r) { return r.json(); })
            .then(function(data) { searchIndex = data; })
            .catch(function(e) { console.warn('Failed to load search index:', e); });
    }

    // Bigram generation for fuzzy matching
    function bigrams(str) {
        var s = str.toLowerCase();
        var set = {};
        for (var i = 0; i < s.length - 1; i++) {
            set[s.substring(i, i + 2)] = true;
        }
        return set;
    }

    function bigramSimilarity(a, b) {
        var ba = bigrams(a);
        var bb = bigrams(b);
        var intersection = 0;
        var union = 0;
        for (var k in ba) { union++; if (bb[k]) intersection++; }
        for (var k2 in bb) { if (!ba[k2]) union++; }
        return union === 0 ? 0 : intersection / union;
    }

    function highlightMatch(text, query) {
        if (!query) return text;
        var idx = text.toLowerCase().indexOf(query.toLowerCase());
        if (idx < 0) return text;
        return text.slice(0, idx) + '<mark>' + text.slice(idx, idx + query.length) + '</mark>' + text.slice(idx + query.length);
    }

    function search(query) {
        if (!searchIndex || !query) return [];
        var q = query.toLowerCase();
        var results = [];

        for (var i = 0; i < searchIndex.length; i++) {
            var item = searchIndex[i];
            var score = 0;

            // Title matching
            if (item.title.toLowerCase().indexOf(q) >= 0) score += 20;
            else if (bigramSimilarity(item.title, query) > 0.3) score += 12;

            // Section-level matching
            var matchedSections = [];
            if (item.sections) {
                for (var s = 0; s < item.sections.length; s++) {
                    var sec = item.sections[s];
                    var secScore = 0;
                    if (sec.heading.toLowerCase().indexOf(q) >= 0) secScore += 8;
                    else if (bigramSimilarity(sec.heading, query) > 0.3) secScore += 5;
                    if (sec.content.toLowerCase().indexOf(q) >= 0) secScore += 2;
                    else if (bigramSimilarity(sec.content.substring(0, 200), query) > 0.2) secScore += 1;
                    if (secScore > 0) {
                        matchedSections.push({ section: sec, score: secScore });
                        score += secScore;
                    }
                }
            }

            // Content matching (fallback)
            if (score === 0) {
                if (item.content.toLowerCase().indexOf(q) >= 0) score += 2;
                else if (item.headers && item.headers.some(function(h) { return h.toLowerCase().indexOf(q) >= 0; })) score += 5;
            }

            if (score > 0) {
                results.push({
                    item: item,
                    score: score,
                    sections: matchedSections
                });
            }
        }

        results.sort(function(a, b) { return b.score - a.score; });
        return results.slice(0, 10);
    }

    function renderResults(scored, query) {
        focusedIdx = -1;
        if (scored.length === 0) {
            results.innerHTML = '<div class="search-result-item"><div class="search-result-title">No results</div></div>';
        } else {
            var html = '';
            var idx = 0;
            for (var i = 0; i < scored.length; i++) {
                var item = scored[i].item;
                var sections = scored[i].sections;

                if (sections.length > 0) {
                    // Show section-level results
                    for (var s = 0; s < Math.min(sections.length, 2); s++) {
                        var sec = sections[s].section;
                        var link = item.route_path + '#' + sec.anchor;
                        var preview = getPreview(sec.content, query);
                        html += '<a href="' + link + '" class="search-result-item" data-idx="' + idx + '" style="display:block;color:inherit;text-decoration:none">';
                        html += '<div class="search-result-title">' + highlightMatch(item.title, query) + '</div>';
                        html += '<div class="search-result-section">' + highlightMatch(sec.heading, query) + '</div>';
                        if (preview) html += '<div class="search-result-preview">' + preview + '</div>';
                        html += '</a>';
                        idx++;
                    }
                } else {
                    // Page-level result
                    var preview = getPreview(item.content, query);
                    html += '<a href="' + item.route_path + '" class="search-result-item" data-idx="' + idx + '" style="display:block;color:inherit;text-decoration:none">';
                    html += '<div class="search-result-title">' + highlightMatch(item.title, query) + '</div>';
                    if (preview) html += '<div class="search-result-preview">' + preview + '</div>';
                    html += '</a>';
                    idx++;
                }
            }
            results.innerHTML = html;
        }
        results.classList.add('active');
    }

    function getPreview(content, query) {
        var q = query.toLowerCase();
        var idx = content.toLowerCase().indexOf(q);
        if (idx < 0) return '';
        var start = Math.max(0, idx - 40);
        var end = Math.min(content.length, idx + 60);
        var preview = (start > 0 ? '...' : '') + content.slice(start, end) + (end < content.length ? '...' : '');
        return highlightMatch(preview, query);
    }

    function setFocus(idx) {
        var items = results.querySelectorAll('.search-result-item[data-idx]');
        items.forEach(function(el) { el.classList.remove('focused'); });
        if (idx >= 0 && idx < items.length) {
            focusedIdx = idx;
            items[idx].classList.add('focused');
            items[idx].scrollIntoView({ block: 'nearest' });
        }
    }

    input.addEventListener('focus', function() { loadIndex(); });

    input.addEventListener('input', function() {
        clearTimeout(debounceTimer);
        debounceTimer = setTimeout(function() {
            var q = input.value.trim();
            if (!q) {
                results.classList.remove('active');
                results.innerHTML = '';
                return;
            }
            var items = search(q);
            renderResults(items, q);
        }, 150);
    });

    input.addEventListener('keydown', function(e) {
        var items = results.querySelectorAll('.search-result-item[data-idx]');
        var count = items.length;

        if (e.key === 'ArrowDown') {
            e.preventDefault();
            setFocus(focusedIdx < count - 1 ? focusedIdx + 1 : 0);
        } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            setFocus(focusedIdx > 0 ? focusedIdx - 1 : count - 1);
        } else if (e.key === 'Enter' && focusedIdx >= 0) {
            e.preventDefault();
            var focused = items[focusedIdx];
            if (focused) window.location.href = focused.getAttribute('href');
        } else if (e.key === 'Escape') {
            results.classList.remove('active');
            input.blur();
        }
    });

    // Global keyboard shortcut: / to focus search
    document.addEventListener('keydown', function(e) {
        if (e.key === '/' && document.activeElement !== input && !['INPUT', 'TEXTAREA'].includes(document.activeElement.tagName)) {
            e.preventDefault();
            input.focus();
        }
    });

    document.addEventListener('click', function(e) {
        if (!e.target.closest('.search-box')) {
            results.classList.remove('active');
        }
    });
})();

// ========== Copy Button Feedback ==========
document.addEventListener('click', function(e) {
    var btn = e.target.closest('.copy-btn');
    if (btn) {
        var orig = btn.textContent;
        btn.textContent = 'Copied!';
        setTimeout(function() { btn.textContent = orig; }, 1500);
    }
});

// ========== Copy Heading Link ==========
document.addEventListener('click', function(e) {
    var anchor = e.target.closest('.header-anchor');
    if (!anchor) return;
    e.preventDefault();
    var url = window.location.origin + window.location.pathname + anchor.getAttribute('href');
    navigator.clipboard.writeText(url).then(function() {
        var tooltip = document.createElement('span');
        tooltip.textContent = 'Link copied!';
        tooltip.style.cssText = 'position:absolute;background:var(--bg-tertiary,#333);color:var(--text-primary,#fff);padding:4px 8px;border-radius:4px;font-size:12px;white-space:nowrap;pointer-events:none;z-index:1000;';
        anchor.style.position = 'relative';
        anchor.appendChild(tooltip);
        setTimeout(function() { tooltip.remove(); }, 1500);
    });
});

// ========== Tabs ==========
document.addEventListener('click', function(e) {
    var btn = e.target.closest('.tab-btn');
    if (!btn) return;
    var container = btn.closest('.tabs-container');
    if (!container) return;

    var tabId = btn.dataset.tab;

    // Update buttons
    container.querySelectorAll('.tab-btn').forEach(function(b) { b.classList.remove('active'); });
    btn.classList.add('active');

    // Update panels
    container.querySelectorAll('.tab-panel').forEach(function(p) {
        p.classList.toggle('active', p.dataset.tab === tabId);
    });
});

// ========== Image Zoom ==========
document.addEventListener('click', function(e) {
    var img = e.target.closest('img.zoomable');
    if (!img) return;

    var overlay = document.createElement('div');
    overlay.className = 'image-zoom-overlay';
    var zoomed = document.createElement('img');
    zoomed.src = img.src;
    zoomed.alt = img.alt;
    overlay.appendChild(zoomed);
    document.body.appendChild(overlay);

    overlay.addEventListener('click', function() { overlay.remove(); });
    document.addEventListener('keydown', function handler(e) {
        if (e.key === 'Escape') {
            overlay.remove();
            document.removeEventListener('keydown', handler);
        }
    });
});

// ========== Back to Top ==========
(function() {
    var btn = document.getElementById('back-to-top');
    if (!btn) return;

    window.addEventListener('scroll', function() {
        btn.classList.toggle('visible', window.scrollY > 300);
    }, { passive: true });
})();
