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
        var meta = document.querySelector('meta[name="novel-base"]');
        var base = (meta && meta.content) ? meta.content : '/';
        if (!base.endsWith('/')) base += '/';
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

    function clearNode(el) {
        while (el.firstChild) el.removeChild(el.firstChild);
    }

    function asSearchText(value) {
        return value == null ? '' : String(value);
    }

    function appendHighlighted(parent, text, query) {
        text = asSearchText(text);
        query = asSearchText(query);
        if (!query) {
            parent.appendChild(document.createTextNode(text));
            return;
        }
        var idx = text.toLowerCase().indexOf(query.toLowerCase());
        if (idx < 0) {
            parent.appendChild(document.createTextNode(text));
            return;
        }
        parent.appendChild(document.createTextNode(text.slice(0, idx)));
        var mark = document.createElement('mark');
        mark.textContent = text.slice(idx, idx + query.length);
        parent.appendChild(mark);
        parent.appendChild(document.createTextNode(text.slice(idx + query.length)));
    }

    function safeSearchHref(routePath, anchor) {
        var path = asSearchText(routePath);
        if (path.charAt(0) !== '/' || path.indexOf('//') === 0 || /[\u0000-\u001f\u007f]/.test(path)) {
            return '#';
        }
        if (anchor) {
            path += '#' + encodeURIComponent(asSearchText(anchor).replace(/[\u0000-\u001f\u007f#\s]/g, ''));
        }
        return path;
    }

    function resultLink(href, idx) {
        var link = document.createElement('a');
        link.href = href;
        link.className = 'search-result-item';
        link.dataset.idx = String(idx);
        link.style.display = 'block';
        link.style.color = 'inherit';
        link.style.textDecoration = 'none';
        return link;
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
        clearNode(results);
        if (scored.length === 0) {
            var empty = document.createElement('div');
            empty.className = 'search-result-item';
            var title = document.createElement('div');
            title.className = 'search-result-title';
            title.textContent = 'No results';
            empty.appendChild(title);
            results.appendChild(empty);
        } else {
            var fragment = document.createDocumentFragment();
            var idx = 0;
            for (var i = 0; i < scored.length; i++) {
                var item = scored[i].item;
                var sections = scored[i].sections;

                if (sections.length > 0) {
                    // Show section-level results
                    for (var s = 0; s < Math.min(sections.length, 2); s++) {
                        var sec = sections[s].section;
                        var preview = getPreview(sec.content, query);
                        var sectionLink = resultLink(safeSearchHref(item.route_path, sec.anchor), idx);
                        var sectionTitle = document.createElement('div');
                        sectionTitle.className = 'search-result-title';
                        appendHighlighted(sectionTitle, item.title, query);
                        sectionLink.appendChild(sectionTitle);
                        var sectionHeading = document.createElement('div');
                        sectionHeading.className = 'search-result-section';
                        appendHighlighted(sectionHeading, sec.heading, query);
                        sectionLink.appendChild(sectionHeading);
                        if (preview) {
                            var sectionPreview = document.createElement('div');
                            sectionPreview.className = 'search-result-preview';
                            appendHighlighted(sectionPreview, preview, query);
                            sectionLink.appendChild(sectionPreview);
                        }
                        fragment.appendChild(sectionLink);
                        idx++;
                    }
                } else {
                    // Page-level result
                    var preview = getPreview(item.content, query);
                    var pageLink = resultLink(safeSearchHref(item.route_path), idx);
                    var pageTitle = document.createElement('div');
                    pageTitle.className = 'search-result-title';
                    appendHighlighted(pageTitle, item.title, query);
                    pageLink.appendChild(pageTitle);
                    if (preview) {
                        var pagePreview = document.createElement('div');
                        pagePreview.className = 'search-result-preview';
                        appendHighlighted(pagePreview, preview, query);
                        pageLink.appendChild(pagePreview);
                    }
                    fragment.appendChild(pageLink);
                    idx++;
                }
            }
            results.appendChild(fragment);
        }
        results.classList.add('active');
    }

    function getPreview(content, query) {
        content = asSearchText(content);
        var q = query.toLowerCase();
        var idx = content.toLowerCase().indexOf(q);
        if (idx < 0) return '';
        var start = Math.max(0, idx - 40);
        var end = Math.min(content.length, idx + 60);
        var preview = (start > 0 ? '...' : '') + content.slice(start, end) + (end < content.length ? '...' : '');
        return preview;
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
                clearNode(results);
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

// ========== Tabs (WAI-ARIA Tabs Pattern) ==========
// Supports both `::: tabs` (class .tabs-container) and `::: code-group`
// (class .code-group). Keyboard navigation: Left/Right/Home/End cycle tabs
// inside the same tablist.

function _novelTabContainer(el) {
    return el.closest('.tabs-container, .code-group');
}

function _novelActivateTab(container, btn) {
    var tabId = btn.dataset.tab;
    container.querySelectorAll('.tab-btn').forEach(function(b) {
        var isActive = b === btn;
        b.classList.toggle('active', isActive);
        b.setAttribute('aria-selected', isActive ? 'true' : 'false');
        b.setAttribute('tabindex', isActive ? '0' : '-1');
    });
    container.querySelectorAll('.tab-panel').forEach(function(p) {
        var isActive = p.dataset.tab === tabId;
        p.classList.toggle('active', isActive);
        if (isActive) {
            p.removeAttribute('hidden');
        } else {
            p.setAttribute('hidden', '');
        }
    });
}

document.addEventListener('click', function(e) {
    var btn = e.target.closest('.tab-btn');
    if (!btn) return;
    var container = _novelTabContainer(btn);
    if (!container) return;
    _novelActivateTab(container, btn);
});

document.addEventListener('keydown', function(e) {
    var btn = e.target.closest('.tab-btn');
    if (!btn) return;
    var container = _novelTabContainer(btn);
    if (!container) return;
    var tabs = Array.from(container.querySelectorAll('.tab-btn'));
    var idx = tabs.indexOf(btn);
    if (idx < 0) return;
    var next = null;
    switch (e.key) {
        case 'ArrowLeft':  next = tabs[(idx - 1 + tabs.length) % tabs.length]; break;
        case 'ArrowRight': next = tabs[(idx + 1) % tabs.length]; break;
        case 'Home':       next = tabs[0]; break;
        case 'End':        next = tabs[tabs.length - 1]; break;
        default: return;
    }
    e.preventDefault();
    _novelActivateTab(container, next);
    next.focus();
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

    function onKey(ev) {
        if (ev.key === 'Escape') close();
    }
    function close() {
        overlay.remove();
        document.removeEventListener('keydown', onKey);
    }
    overlay.addEventListener('click', close);
    document.addEventListener('keydown', onKey);
});

// ========== Back to Top ==========
(function() {
    var btn = document.getElementById('back-to-top');
    if (!btn) return;

    window.addEventListener('scroll', function() {
        btn.classList.toggle('visible', window.scrollY > 300);
    }, { passive: true });
})();

// ========== Page Feedback ==========
(function() {
    var widgets = document.querySelectorAll('.page-feedback');
    if (widgets.length === 0) return;

    widgets.forEach(function(widget) {
        var key = 'novel-feedback:' + (widget.dataset.feedbackKey || location.pathname);
        var thanks = widget.querySelector('.page-feedback-thanks');
        var actions = widget.querySelector('.page-feedback-actions');

        if (localStorage.getItem(key)) {
            if (actions) actions.hidden = true;
            if (thanks) thanks.hidden = false;
        }

        widget.addEventListener('click', function(e) {
            var btn = e.target.closest('[data-feedback-value]');
            if (!btn) return;
            try {
                localStorage.setItem(key, btn.dataset.feedbackValue || 'sent');
            } catch (_) {}
            if (actions) actions.hidden = true;
            if (thanks) thanks.hidden = false;
        });
    });
})();
