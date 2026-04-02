/// Injected JavaScript for interactive SVG charts.
pub const JS: &str = r#"
(function() {
  var svg = document.currentScript
    ? document.currentScript.closest('svg')
    : document.querySelector('svg[data-xmin]');
  if (!svg) return;

  // ── coordinate readout ──────────────────────────────────────────────
  var readout = svg.getElementById('kuva-readout');
  var xmin = parseFloat(svg.getAttribute('data-xmin') || 'NaN');
  var xmax = parseFloat(svg.getAttribute('data-xmax') || 'NaN');
  var ymin = parseFloat(svg.getAttribute('data-ymin') || 'NaN');
  var ymax = parseFloat(svg.getAttribute('data-ymax') || 'NaN');
  var pl   = parseFloat(svg.getAttribute('data-plot-left')   || 'NaN');
  var pt   = parseFloat(svg.getAttribute('data-plot-top')    || 'NaN');
  var pr   = parseFloat(svg.getAttribute('data-plot-right')  || 'NaN');
  var pb   = parseFloat(svg.getAttribute('data-plot-bottom') || 'NaN');
  var logX = svg.getAttribute('data-log-x') === '1';
  var logY = svg.getAttribute('data-log-y') === '1';
  var hasAxes = !isNaN(xmin);

  function px2data(px, py) {
    if (!hasAxes) return null;
    var fx = (px - pl) / (pr - pl);
    var fy = (py - pt) / (pb - pt);
    var x = logX ? Math.pow(10, xmin + fx * (xmax - xmin)) : xmin + fx * (xmax - xmin);
    var y = logY ? Math.pow(10, ymax - fy * (ymax - ymin)) : ymax - fy * (ymax - ymin);
    return { x: x, y: y };
  }

  function fmt(v) {
    if (Math.abs(v) >= 1e4 || (v !== 0 && Math.abs(v) < 0.01)) return v.toExponential(2);
    return parseFloat(v.toFixed(3)).toString();
  }

  if (readout && hasAxes) {
    svg.addEventListener('mousemove', function(e) {
      var rect = svg.getBoundingClientRect();
      var scaleX = (svg.viewBox && svg.viewBox.baseVal.width) ? svg.viewBox.baseVal.width / rect.width : 1;
      var scaleY = (svg.viewBox && svg.viewBox.baseVal.height) ? svg.viewBox.baseVal.height / rect.height : 1;
      var px = (e.clientX - rect.left) * scaleX;
      var py = (e.clientY - rect.top)  * scaleY;
      var d = px2data(px, py);
      if (d && px >= pl && px <= pr && py >= pt && py <= pb) {
        readout.setAttribute('x', px + 6);
        readout.setAttribute('y', py - 6);
        readout.textContent = 'x=' + fmt(d.x) + '  y=' + fmt(d.y);
      } else {
        readout.textContent = '';
      }
    });
    svg.addEventListener('mouseleave', function() { readout.textContent = ''; });
  }

  // ── state ────────────────────────────────────────────────────────────
  var ttGroups     = Array.prototype.slice.call(svg.querySelectorAll('g.tt'));
  var legendEntries = Array.prototype.slice.call(svg.querySelectorAll('g.legend-entry'));
  var pinned = null;
  var hiddenGroups = {};  // group-name → true when hidden via legend

  function groupOf(g) { return g.getAttribute('data-group'); }
  function isMuted(g) { var gr = groupOf(g); return gr && hiddenGroups[gr]; }

  // ── highlight helpers ────────────────────────────────────────────────
  // Only dim/undim groups that are currently visible (not legend-hidden).
  // Hover dim: only affects non-muted groups.
  function highlight(g) {
    ttGroups.forEach(function(other) {
      if (other !== g && !isMuted(other)) other.classList.add('dim');
    });
    g.classList.remove('dim');
  }

  function unhighlight() {
    ttGroups.forEach(function(g) {
      if (!isMuted(g)) g.classList.remove('dim');
    });
  }

  // ── clear-all ────────────────────────────────────────────────────────
  function clearAll() {
    if (pinned) { pinned.classList.remove('pinned'); pinned = null; }
    unhighlight();
    // Unmute all legend-muted series.
    var anyMuted = false;
    for (var k in hiddenGroups) { if (hiddenGroups[k]) { anyMuted = true; break; } }
    if (anyMuted) {
      legendEntries.forEach(function(le) { le.style.opacity = ''; });
      ttGroups.forEach(function(g) { g.classList.remove('muted'); });
      hiddenGroups = {};
    }
    // Clear search.
    var inp = svg.querySelector('foreignObject input');
    if (inp && inp.value) {
      inp.value = '';
      ttGroups.forEach(function(g) { g.classList.remove('dim'); });
    }
  }

  // ── point click / hover ──────────────────────────────────────────────
  ttGroups.forEach(function(g) {
    g.addEventListener('mouseenter', function() {
      if (pinned) return;
      highlight(g);
    });
    g.addEventListener('mouseleave', function() {
      if (pinned) return;
      unhighlight();
    });
    g.addEventListener('click', function(e) {
      e.stopPropagation();
      if (pinned === g) {
        g.classList.remove('pinned'); pinned = null; unhighlight();
      } else {
        if (pinned) pinned.classList.remove('pinned');
        pinned = g;
        g.classList.add('pinned');
        highlight(g);
      }
    });
  });

  // ── legend toggle ────────────────────────────────────────────────────
  legendEntries.forEach(function(le) {
    le.addEventListener('click', function(e) {
      e.stopPropagation();
      var group = groupOf(le);
      if (!group) return;
      if (hiddenGroups[group]) {
        // Unmute this group.
        delete hiddenGroups[group];
        le.style.opacity = '';
        ttGroups.forEach(function(g) {
          if (groupOf(g) === group) g.classList.remove('muted');
        });
      } else {
        // Mute this group (semi-transparent, still visible).
        hiddenGroups[group] = true;
        le.style.opacity = '0.35';
        ttGroups.forEach(function(g) {
          if (groupOf(g) === group) { g.classList.add('muted'); g.classList.remove('dim'); }
        });
      }
    });
  });

  // ── search ───────────────────────────────────────────────────────────
  var inp = svg.querySelector('foreignObject input');
  if (inp) {
    inp.addEventListener('input', function() {
      var q = inp.value.trim().toLowerCase();
      ttGroups.forEach(function(g) {
        if (isMuted(g)) return;
        if (!q) { g.classList.remove('dim'); return; }
        var title = g.querySelector('title');
        var text = [(title ? title.textContent : ''), groupOf(g) || '',
                    g.getAttribute('data-x') || '', g.getAttribute('data-y') || ''].join(' ');
        g.classList.toggle('dim', text.toLowerCase().indexOf(q) < 0);
      });
    });
    inp.addEventListener('click', function(e) { e.stopPropagation(); });
  }

  // ── save SVG ─────────────────────────────────────────────────────────
  var btn = svg.getElementById('kuva-save-btn');
  if (btn) {
    btn.addEventListener('click', function(e) {
      e.stopPropagation();
      var s = new XMLSerializer().serializeToString(svg);
      var blob = new Blob([s], { type: 'image/svg+xml' });
      var url = URL.createObjectURL(blob);
      var a = document.createElement('a'); a.href = url; a.download = 'kuva-chart.svg'; a.click();
      URL.revokeObjectURL(url);
    });
  }

  // ── click anywhere (on the SVG but not on a point/legend/UI) → clear ─
  document.addEventListener('click', function(e) {
    if (!svg.contains(e.target)) return;
    if (e.target.closest('g.tt') || e.target.closest('g.legend-entry') || e.target.closest('foreignObject')) return;
    clearAll();
  });

  // ── Escape → clear ───────────────────────────────────────────────────
  document.addEventListener('keydown', function(e) {
    if (e.key === 'Escape') clearAll();
  });

  // ── volcano threshold dragging ────────────────────────────────────────
  svg.querySelectorAll('g.kuva-threshold').forEach(function(g) {
    if (g.getAttribute('data-axis') !== 'y') return;
    var dragging = false, startY = 0, startVal = 0;
    g.style.cursor = 'ns-resize';
    g.addEventListener('mousedown', function(e) {
      dragging = true; startY = e.clientY; startVal = parseFloat(g.getAttribute('data-value') || 0);
      e.preventDefault(); e.stopPropagation();
    });
    document.addEventListener('mousemove', function(e) {
      if (!dragging) return;
      var rect = svg.getBoundingClientRect();
      var scaleY = (svg.viewBox && svg.viewBox.baseVal.height) ? svg.viewBox.baseVal.height / rect.height : 1;
      var newVal = startVal - (e.clientY - startY) * scaleY * (ymax - ymin) / (pb - pt);
      g.setAttribute('data-value', newVal);
      var line = g.querySelector('line');
      if (line) {
        var newY = pt + (ymax - newVal) / (ymax - ymin) * (pb - pt);
        line.setAttribute('y1', newY); line.setAttribute('y2', newY);
      }
      var lfc_g = svg.querySelector('g.kuva-threshold[data-axis="x"]');
      var lfc_thresh = lfc_g ? parseFloat(lfc_g.getAttribute('data-value') || 1) : 1;
      ttGroups.forEach(function(pt_g) {
        var lfc = parseFloat(pt_g.getAttribute('data-logfc') || 'NaN');
        var nlp = parseFloat(pt_g.getAttribute('data-pvalue') || 'NaN');
        if (isNaN(lfc) || isNaN(nlp)) return;
        var sig = nlp >= newVal;
        var circle = pt_g.querySelector('circle');
        if (circle) circle.setAttribute('fill', sig && lfc >= lfc_thresh ? '#d73027' : sig && lfc <= -lfc_thresh ? '#4575b4' : '#aaaaaa');
      });
    });
    document.addEventListener('mouseup', function() { dragging = false; });
  });
})();
"#;
