// Custom JavaScript for Rust Patterns book

// Highlight diamond separators
document.addEventListener('DOMContentLoaded', function() {
    // Find and style diamond separators
    const paragraphs = document.querySelectorAll('.content p');
    paragraphs.forEach(p => {
        if (p.textContent.trim() === '◆ ◆ ◆') {
            p.classList.add('diamond-separator');
            p.style.textAlign = 'center';
            p.style.fontSize = '1.5em';
            p.style.letterSpacing = '1em';
            p.style.color = '#888';
            p.style.margin = '2em 0';
        }
    });

    // Add pattern number badges
    const h1s = document.querySelectorAll('.content h1');
    h1s.forEach(h1 => {
        const match = h1.textContent.match(/^(\d+)\.\s+/);
        if (match) {
            const num = match[1];
            const badge = document.createElement('span');
            badge.className = 'pattern-badge';
            badge.textContent = '#' + num;
            badge.style.cssText = `
                background: #a72145;
                color: white;
                padding: 0.2em 0.5em;
                border-radius: 4px;
                font-size: 0.6em;
                margin-right: 0.5em;
                vertical-align: middle;
            `;
            h1.insertBefore(badge, h1.firstChild);
            h1.textContent = h1.textContent.replace(/^\d+\.\s+/, '');
        }
    });
});
