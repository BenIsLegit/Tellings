// @ts-check
/*
  Draws one frame of a prayer video from a plan.

  The same file draws the live preview in the app, the stills that become the
  video, and the tap-through guide. It decides nothing: which prayer, which
  bead, which voice and which colors all come from the plan the engine made.

  A plain script on purpose (no build step, no imports), so the engine can
  drop it into a page as it is.
*/
(function (root) {
  "use strict";

  /** @param {string} tag @param {string} [cls] @returns {HTMLElement} */
  function el(tag, cls) {
    const e = document.createElement(tag);
    if (cls) e.className = cls;
    return e;
  }
  /** @param {unknown} s */
  function esc(s) {
    return String(s).replace(/[&<>"]/g, function (c) {
      return c === "&" ? "&amp;" : c === "<" ? "&lt;" : c === ">" ? "&gt;" : "&quot;";
    });
  }
  /** @param {number} n */
  function r1(n) {
    return Math.round(n * 100) / 100;
  }

  // ───────────────────────── where things sit ─────────────────────────

  /**
   * The first video's layout, scaled to any screen. On 2076 x 2152 this returns
   * the first video's own numbers.
   * @param {any} stage
   * @param {boolean} [pictureBeside]  true when some scene puts its picture beside the words
   */
  function metrics(stage, pictureBeside) {
    const safe = stage.safe || { top: 0, right: 0, bottom: 0, left: 0 };
    const W = stage.width - safe.left - safe.right;
    const H = stage.height - safe.top - safe.bottom;
    const tall = W / H < 0.8;
    /** @type {any} */
    let m;
    if (!tall) {
      const u = Math.min(W / 2076, H / 2152);
      const baseCol = 816 * u;
      const free = W - 1260 * u;
      const oy = (H - 2152 * u) / 2;
      if (pictureBeside && W / H >= 1.45) {
        // A wide screen has room for three things in a row: beads, words, picture.
        const colW = free / 2 - 30 * u;
        const sx = 1180 * u + colW + 60 * u;
        const sw = W - sx - 80 * u;
        const sh = Math.min(sw * 1.1, 1500 * u);
        m = {
          u: u,
          b: u,
          t: u * Math.min(1.3, Math.sqrt(colW / baseCol)),
          beads: { x: 60 * u, y: oy + 230 * u, w: 1040 * u, h: 1657 * u },
          col: { x: 1180 * u, y: oy + 250 * u, w: colW, h: 1700 * u },
          side: { x: sx, y: oy + 250 * u + (1700 * u - sh) / 2, w: sw, h: sh },
        };
      } else {
        const colW = Math.min(free, baseCol * 1.7);
        const ox = (free - colW) / 2;
        m = {
          u: u,
          b: u,
          t: u * Math.min(1.3, Math.sqrt(colW / baseCol)),
          beads: { x: ox + 60 * u, y: oy + 230 * u, w: 1040 * u, h: 1657 * u },
          col: { x: ox + 1180 * u, y: oy + 250 * u, w: colW, h: 1700 * u },
        };
      }
    } else {
      // A tall screen: beads on top, words below. The words get the bigger share.
      const u = W / 1240;
      const top = 60 * u;
      const bh = Math.min(H * 0.4, (W - 120 * u) / (300 / 478));
      const bw = bh * (300 / 478);
      const colY = top + bh + 40 * u;
      m = {
        u: u,
        b: bw / 1040,
        t: u * 0.74,
        beads: { x: (W - bw) / 2, y: top, w: bw, h: bh },
        col: { x: 80 * u, y: colY, w: W - 160 * u, h: H - colY - 190 * u },
      };
    }
    m.inner = { x: safe.left, y: safe.top, w: W, h: H };
    m.tall = tall;
    return m;
  }

  // ───────────────────────── the beads ─────────────────────────

  /** @param {number} cx @param {number} cy @param {number} r */
  function circlePath(cx, cy, r) {
    return "M" + r1(cx - r) + " " + r1(cy) + "a" + r + " " + r + " 0 1 0 " + 2 * r + " 0a" + r + " " + r + " 0 1 0 " + -2 * r + " 0Z";
  }

  /**
   * What hangs at the end. Each shape is drawn around (0,0), 58 units tall, like
   * the first video's cross: top at -29, bottom at 29.
   * @param {string} style
   * @returns {{d: string, inlay?: string, ring?: {cx: number, cy: number, r: number}}}
   */
  function crossShape(style) {
    const latin = "M-4 -29H4V-15H18V-7H4V29H-4V-7H-18V-15H-4Z";
    switch (style) {
      case "st-benedict":
        // A Latin cross with the round St. Benedict medal where the arms meet.
        return { d: latin + circlePath(0, -11, 8.5), inlay: circlePath(0, -11, 5) + "M0 -14.5V-7.5M-3.5 -11H3.5" };
      case "orthodox":
        // Three bars: the title board, the arms, and the slanted footrest.
        return { d: "M-3.5 -29H3.5V29H-3.5ZM-8 -24H8V-19H-8ZM-17 -13H17V-7H-17ZM-11 7L11 14V19.5L-11 12.5Z" };
      case "greek":
        return { d: "M-4.5 -29H4.5V-13.5H20V-4.5H4.5V11H-4.5V-4.5H-20V-13.5H-4.5Z" };
      case "celtic":
        return { d: latin, ring: { cx: 0, cy: -11, r: 12 } };
      case "san-damiano":
        return { d: "M-8 -29H8V-21H5V-15H13V-18H19V-4H13V-7H7V22H10V29H-10V22H-7V-7H-13V-4H-19V-18H-13V-15H-5V-21H-8Z" };
      case "knotted":
        // Knots of wool tied in a cross, as on a prayer rope.
        return { d: [[0, -24], [0, -13], [-11, -13], [11, -13], [0, -2], [0, 9]].map(function (p) { return circlePath(p[0], p[1], 5.2); }).join("") + "M-1.5 14L-7 29H7L1.5 14Z" };
      case "tassel":
        return { d: circlePath(0, -23, 5.5) + "M-3 -18H3L10 29H-10Z", inlay: "M-5 -6H5M-4 6V27M0 6V28M4 6V27" };
      case "medal":
        return { d: circlePath(0, -14, 14), inlay: "M0 -22V-6M-8 -14H8" };
      default:
        return { d: latin };
    }
  }

  /** @param {string} style @param {number} x @param {number} y */
  function medalMarkup(style, x, y) {
    let shape, glow, halo, inlay;
    if (style === "miraculous") {
      // An oval medal with the M and the cross above it.
      shape = '<ellipse class="dot" cx="' + x + '" cy="' + y + '" rx="10.5" ry="13"/>';
      glow = '<ellipse class="glow" cx="' + x + '" cy="' + y + '" rx="20" ry="22.5"/>';
      halo = '<ellipse class="halo" cx="' + x + '" cy="' + y + '" rx="15" ry="17.5"/>';
      inlay = "M" + (x - 5) + " " + (y + 6.5) + "V" + (y - 0.5) + "L" + x + " " + (y + 4) + "L" + (x + 5) + " " + (y - 0.5) + "V" + (y + 6.5) + "M" + x + " " + (y - 1.5) + "V" + (y - 8.5) + "M" + (x - 3) + " " + (y - 6) + "H" + (x + 3);
    } else {
      shape = '<circle class="dot" cx="' + x + '" cy="' + y + '" r="12"/>';
      glow = '<circle class="glow" cx="' + x + '" cy="' + y + '" r="22"/>';
      halo = '<circle class="halo" cx="' + x + '" cy="' + y + '" r="16.5"/>';
      // The plain medal is the first video's: a circle with a small cross.
      inlay = "M" + x + " " + (y - 6.5) + "V" + (y + 6.5) + "M" + (x - 6.5) + " " + y + "H" + (x + 6.5);
      if (style === "st-benedict") inlay = circlePath(x, y, 8.8) + "M" + x + " " + (y - 8.8) + "V" + (y + 8.8) + "M" + (x - 8.8) + " " + y + "H" + (x + 8.8);
    }
    return '<g class="bead medal" data-bead="medal">' + glow + halo + shape + '<path class="inlay" d="' + inlay + '"/></g>';
  }

  const MATERIALS = {
    wood: { color: "#9A6B43", light: 22, dark: 42 },
    glass: { color: "#6FA8DC", light: 62, dark: 30 },
    stone: { color: "#8D939E", light: 16, dark: 38 },
    pearl: { color: "#EFE9DD", light: 70, dark: 26 },
    metal: { color: "#C2C7CF", light: 80, dark: 52 },
    knots: { color: "#4A5163", light: 10, dark: 30 },
  };

  /** @param {SVGSVGElement} svg @param {any} stage */
  function drawBeads(svg, stage) {
    const d = stage.drawing;
    const look = stage.beads;
    const k = look.size || 1;
    const mat = MATERIALS[/** @type {keyof typeof MATERIALS} */ (look.style)];
    svg.setAttribute("viewBox", d.viewBox.join(" "));
    svg.setAttribute("class", "pvc-beads" + (mat ? " mat" : "") + (look.cord === "cord" ? " cord" : ""));
    let h = "";
    if (mat) {
      h += '<defs><radialGradient id="pvc-mat" cx="34%" cy="30%" r="78%">' +
        '<stop offset="0" style="stop-color:color-mix(in srgb, var(--bead) ' + (100 - mat.light) + '%, white)"/>' +
        '<stop offset=".5" style="stop-color:var(--bead)"/>' +
        '<stop offset="1" style="stop-color:color-mix(in srgb, var(--bead) ' + (100 - mat.dark) + '%, black)"/>' +
        "</radialGradient></defs>";
    }
    h += '<circle class="chain" cx="' + d.loopCx + '" cy="' + d.loopCy + '" r="' + d.loopR + '"/>' +
      '<line class="chain" x1="' + d.strand[0] + '" y1="' + d.strand[1] + '" x2="' + d.strand[0] + '" y2="' + d.strand[2] + '"/>';
    d.spots.forEach(function (/** @type {any} */ p) {
      const id = esc(p.id);
      if (p.kind === "small" || p.kind === "big") {
        const r = r1((p.kind === "small" ? 5 : 7.5) * k);
        h += '<g class="bead round" data-bead="' + id + '">' +
          '<circle class="glow" cx="' + p.x + '" cy="' + p.y + '" r="' + r1(r + 9) + '"/>' +
          '<circle class="halo" cx="' + p.x + '" cy="' + p.y + '" r="' + r1(r + 4.5) + '"/>' +
          '<circle class="dot" cx="' + p.x + '" cy="' + p.y + '" r="' + r + '"/>';
        if (look.style === "knots") {
          const q = r1(r * 0.55);
          h += '<path class="knot" d="M' + r1(p.x - q) + " " + r1(p.y - q) + "L" + r1(p.x + q) + " " + r1(p.y + q) + "M" + r1(p.x + q) + " " + r1(p.y - q) + "L" + r1(p.x - q) + " " + r1(p.y + q) + '"/>';
        }
        h += "</g>";
      } else if (p.kind === "gap") {
        h += '<g class="bead gap" data-bead="' + id + '">' +
          '<circle class="glow" cx="' + p.x + '" cy="' + p.y + '" r="12"/>' +
          '<circle class="halo" cx="' + p.x + '" cy="' + p.y + '" r="7.5"/>' +
          '<rect class="dot" x="' + (p.x - 3) + '" y="' + (p.y - 3) + '" width="6" height="6" transform="rotate(45 ' + p.x + " " + p.y + ')"/></g>';
      } else if (p.kind === "medal") {
        h += medalMarkup(look.medal, p.x, p.y);
      } else if (p.kind === "cross") {
        const c = crossShape(look.cross);
        const t = ' transform="translate(' + p.x + " " + p.y + ')"';
        h += '<g class="bead cross" data-bead="' + id + '"' + t + ">" +
          '<path class="halo" d="' + c.d + '"/>' +
          (c.ring ? '<circle class="ring" cx="' + c.ring.cx + '" cy="' + c.ring.cy + '" r="' + c.ring.r + '"/>' : "") +
          '<path class="dot" d="' + c.d + '"/>' +
          (c.inlay ? '<path class="inlay" d="' + c.inlay + '"/>' : "") + "</g>";
      }
    });
    svg.innerHTML = h;
    return mat ? look.color || mat.color : null;
  }

  // ───────────────────────── fonts ─────────────────────────

  /**
   * The four font files the first video used, as CSS. `urls` maps a short name
   * to wherever the file is: a web address in the app, the file itself in a still.
   * @param {{cardo400: string, cardo700: string, fira400: string, fira500: string}} urls
   */
  function fontCss(urls) {
    /** @param {string} family @param {number} weight @param {string} url */
    const face = function (family, weight, url) {
      return '@font-face{font-family:"' + family + '";font-style:normal;font-weight:' + weight + ';font-display:block;src:url("' + url + '") format("woff2");}';
    };
    return face("Cardo", 400, urls.cardo400) + face("Cardo", 700, urls.cardo700) + face("Fira Sans", 400, urls.fira400) + face("Fira Sans", 500, urls.fira500);
  }

  /** Fonts must be declared on the page itself, even when the frame sits in a shadow root. */
  function installFonts(/** @type {string} */ css) {
    if (document.getElementById("pvc-fonts")) return;
    const s = document.createElement("style");
    s.id = "pvc-fonts";
    s.textContent = css;
    document.head.appendChild(s);
  }

  // ───────────────────────── one frame ─────────────────────────

  /**
   * @param {HTMLElement} host   where the frame goes
   * @param {any} plan           the plan from the engine
   * @param {{css?: string, shadow?: boolean, picture?: (file: string) => string | Promise<string>}} [opts]
   */
  function create(host, plan, opts) {
    opts = opts || {};
    const stage = plan.stage;
    const m = metrics(stage, plan.scenes.some(function (/** @type {any} */ s) { return s.picture && s.picture.place === "beside"; }));
    /** @type {HTMLElement | ShadowRoot} */
    const mount = opts.shadow ? host.shadowRoot || host.attachShadow({ mode: "open" }) : host;
    mount.textContent = "";
    if (opts.css) {
      const style = document.createElement("style");
      style.textContent = opts.css;
      mount.appendChild(style);
    }

    const st = el("div", "pvc-stage");
    st.style.width = stage.width + "px";
    st.style.height = stage.height + "px";
    st.style.setProperty("--u", String(m.u));
    st.style.setProperty("--b", String(m.b));
    st.style.setProperty("--t", String(m.t));

    const bg = el("div", "pvc-bg");
    const inner = el("div", "pvc-inner");
    Object.assign(inner.style, { left: m.inner.x + "px", top: m.inner.y + "px", width: m.inner.w + "px", height: m.inner.h + "px" });

    const svg = /** @type {SVGSVGElement} */ (document.createElementNS("http://www.w3.org/2000/svg", "svg"));
    svg.setAttribute("aria-hidden", "true");
    Object.assign(svg.style, { left: m.beads.x + "px", top: m.beads.y + "px", width: m.beads.w + "px", height: m.beads.h + "px" });
    const beadColor = drawBeads(svg, stage);
    if (beadColor) st.style.setProperty("--bead", beadColor);
    if (stage.beads.cordColor) st.style.setProperty("--cord", stage.beads.cordColor);

    // The loop's middle, in stage pixels. The picture and the mystery's name both sit here.
    const scale = m.beads.w / stage.drawing.viewBox[2];
    const cx = m.beads.x + (stage.drawing.loopCx - stage.drawing.viewBox[0]) * scale;
    const cy = m.beads.y + (stage.drawing.loopCy - stage.drawing.viewBox[1]) * scale;
    const artSize = 790 * m.b;
    const art = el("div", "pvc-art");
    Object.assign(art.style, { left: cx - artSize / 2 + "px", top: cy - artSize / 2 + "px", width: artSize + "px", height: artSize + "px" });
    const center = el("div", "pvc-center");
    Object.assign(center.style, { left: cx + "px", top: cy + "px", width: 600 * m.b + "px" });
    const cSet = el("p", "set"), cNum = el("p", "num"), cName = el("p", "name");
    center.append(cSet, cNum, cName);

    const col = el("div", "pvc-col");
    Object.assign(col.style, { left: m.col.x + "px", top: m.col.y + "px", width: m.col.w + "px", height: m.col.h + "px" });
    const side = el("div", "pvc-side");
    const where = el("p", "where"), title = el("h1", "title"), pips = el("div", "pips"), tag = el("p", "tag");
    const ann = el("div", "announce"), prayer = el("p", "prayer"), hint = el("p", "hint");
    col.append(where, title, pips, tag, ann, prayer, hint);
    if (m.side) {
      // Its own place, to the right of the words.
      Object.assign(side.style, { position: "absolute", left: m.side.x + "px", top: m.side.y + "px", width: m.side.w + "px", height: m.side.h + "px" });
    } else {
      // No room beside: the picture sits above the words and takes part of their column.
      side.style.height = Math.round(m.col.h * 0.3) + "px";
      col.prepend(side);
    }

    const credit = el("p", "pvc-credit"), count = el("p", "pvc-count"), track = el("div", "pvc-track"), fill = el("i");
    track.appendChild(fill);
    const pad = 80 * m.u;
    Object.assign(credit.style, { left: pad + "px", bottom: 122 * m.u + "px", maxWidth: m.inner.w - pad * 2 - 616 * m.u + "px" });
    Object.assign(count.style, { right: pad + "px", bottom: 122 * m.u + "px" });
    Object.assign(track.style, { left: pad + "px", right: pad + "px", bottom: 92 * m.u + "px" });

    inner.append(svg, art, center, col, credit, count, track);
    if (m.side) inner.append(side);
    st.append(bg, inner);
    mount.appendChild(st);

    // Pictures are loaded once and kept, so a change of scene never waits on a file twice.
    /** @type {Map<string, Promise<HTMLImageElement | null>>} */
    const images = new Map();
    /** @param {string} file */
    function image(file) {
      let p = images.get(file);
      if (!p) {
        p = Promise.resolve(opts && opts.picture ? opts.picture(file) : file).then(function (src) {
          if (!src) return null;
          const img = new Image();
          img.alt = "";
          img.src = src;
          return img.decode().then(function () { return img; }, function () { return null; });
        }, function () { return null; });
        images.set(file, p);
      }
      return p;
    }

    const fontsReady = (function () {
      /** @type {Set<string>} */
      const families = new Set(["Cardo", "Fira Sans"]);
      plan.scenes.forEach(function (/** @type {any} */ s) { families.add(s.colors.titleFont); families.add(s.colors.textFont); });
      /** @type {Promise<unknown>[]} */
      const loads = [];
      families.forEach(function (f) {
        ["400", "500", "700"].forEach(function (w) { loads.push(document.fonts.load(w + ' 60px "' + f + '"').catch(function () {})); });
      });
      return Promise.all(loads).then(function () { return document.fonts.ready; });
    })();

    /** Shrink the prayer until the column fits, as the first video did. */
    function fit(/** @type {number} */ size) {
      const start = 78 * m.t * size;
      const floor = Math.min(start, 40 * m.t);
      const step = Math.max(1, 2 * m.t);
      let f = start;
      prayer.style.fontSize = r1(f) + "px";
      while (f > floor && col.scrollHeight > col.clientHeight) {
        f = Math.max(floor, f - step);
        prayer.style.fontSize = r1(f) + "px";
      }
      return { font: r1(f), overflow: col.scrollHeight > col.clientHeight };
    }

    let shown = -1;

    /**
     * Draw one scene.
     * @param {number} index            which scene of the plan
     * @param {{line?: number | null}} [o]  the line being spoken, when following along
     */
    function show(index, o) {
      const i = Math.max(0, Math.min(plan.scenes.length - 1, index | 0));
      const sc = plan.scenes[i];
      const line = o && typeof o.line === "number" ? o.line : null;
      shown = i;
      const c = sc.colors;
      const vars = { "--ground": c.background, "--surface": c.panel, "--ink": c.text, "--muted": c.quietText, "--line": c.lines, "--done": c.done, "--brass": c.marker };
      Object.keys(vars).forEach(function (k) { st.style.setProperty(k, /** @type {any} */ (vars)[k]); });
      st.style.setProperty("--display", '"' + c.titleFont + '", "Cardo", "GFS Baskerville", "Crimson Text", Georgia, serif');
      st.style.setProperty("--body", '"' + c.textFont + '", "Fira Sans", "Source Sans 3", "DejaVu Sans", sans-serif');
      st.style.setProperty("--t", String(m.t * (sc.words.size || 1)));

      // The beads: gold for where you are, filled for where you have been.
      svg.querySelectorAll(".bead").forEach(function (g) {
        const id = g.getAttribute("data-bead") || "";
        const now = sc.bead === id;
        const first = stage.firstScene[id];
        const done = sc.kind === "outro" || (sc.kind === "prayer" && first !== undefined && first < i);
        g.classList.toggle("now", now);
        g.classList.toggle("done", !now && done);
      });

      // The words inside the loop.
      cSet.textContent = sc.center.label;
      cNum.hidden = !sc.center.number;
      cNum.textContent = sc.center.number || "";
      cName.textContent = sc.center.name;
      cName.hidden = !sc.center.name;

      // The prayer column.
      where.textContent = sc.place;
      where.hidden = !sc.place;
      title.textContent = sc.title;
      pips.textContent = "";
      pips.hidden = !sc.pip;
      if (sc.pip) {
        for (let k = 1; k <= sc.pip[1]; k++) pips.appendChild(el("span", "pip" + (k < sc.pip[0] ? " done" : k === sc.pip[0] ? " now" : "")));
      }
      tag.hidden = !sc.label;
      tag.textContent = sc.label || "";
      ann.textContent = "";
      ann.hidden = !sc.announce;
      if (sc.announce) {
        const lead = el("p", "small"), say = el("p", "say");
        lead.textContent = sc.announce.lead;
        say.textContent = sc.announce.say;
        ann.append(lead, say);
      }
      prayer.textContent = "";
      prayer.hidden = !sc.showLines;
      prayer.classList.toggle("follow", line !== null);
      if (sc.showLines) {
        sc.lines.forEach(function (/** @type {string} */ text, /** @type {number} */ k) {
          const s = el("span", line === k ? "on" : "");
          s.textContent = text;
          prayer.appendChild(s);
        });
      }
      hint.hidden = !sc.hint;
      hint.textContent = sc.hint || "";

      count.textContent = sc.count;
      fill.style.width = (100 * sc.progress).toFixed(2) + "%";
      count.hidden = track.hidden = !sc.words.progress;

      // The picture goes in the loop, above the words, or behind everything.
      const pic = sc.picture;
      const place = pic ? pic.place : "";
      credit.textContent = pic ? pic.credit : "";
      credit.hidden = !pic || !pic.credit;
      const put = function (/** @type {HTMLImageElement | null} */ img) {
        if (shown !== i) return fit(sc.words.size || 1);
        const boxes = { loop: art, beside: side, background: bg };
        Object.keys(boxes).forEach(function (key) {
          const box = /** @type {any} */ (boxes)[key];
          const on = !!img && place === key;
          box.hidden = !on;
          if (on && img) {
            img.style.objectPosition = pic.focus || "50% 50%";
            img.style.transformOrigin = pic.focus || "50% 50%";
            img.style.transform = pic.zoom && pic.zoom !== 1 ? "scale(" + pic.zoom + ")" : "";
            if (box.firstChild !== img) box.replaceChildren(img);
          }
        });
        center.hidden = !!img && place === "loop";
        return fit(sc.words.size || 1);
      };
      if (!pic) return Promise.resolve(put(null));
      return image(pic.file).then(put);
    }

    art.hidden = side.hidden = bg.hidden = true;

    return {
      el: st,
      plan: plan,
      metrics: m,
      ready: fontsReady,
      show: show,
      /** Load every picture the plan uses. */
      preload: function () {
        /** @type {Promise<unknown>[]} */
        const all = [];
        plan.scenes.forEach(function (/** @type {any} */ s) { if (s.picture) all.push(image(s.picture.file)); });
        return Promise.all(all);
      },
      /** Shrink the frame to fit a box. Returns the scale used. */
      fitTo: function (/** @type {number} */ w, /** @type {number} */ h) {
        const s = Math.min(w / stage.width, h / stage.height);
        st.style.transform = "scale(" + s + ")";
        return s;
      },
    };
  }

  // ───────────────────────── the tap-through guide ─────────────────────────

  /**
   * The whole plan on one page, moved a bead at a time by tap, click or arrow key.
   * @param {any} plan
   * @param {{picture?: (file: string) => string}} [opts]
   */
  function guide(plan, opts) {
    const wrap = el("div", "pvc-guide");
    const fitBox = el("div", "pvc-fit");
    const help = el("p", "pvc-guide-help");
    help.textContent = "Tap the right side to go on. Tap the left side to go back.";
    wrap.append(fitBox);
    document.body.append(wrap, help);
    const frame = create(fitBox, plan, opts);
    let i = Number((location.hash || "").replace("#", "")) || 0;
    const size = function () {
      const s = frame.fitTo(window.innerWidth, window.innerHeight);
      fitBox.style.width = plan.stage.width * s + "px";
      fitBox.style.height = plan.stage.height * s + "px";
    };
    const go = function (/** @type {number} */ to) {
      i = Math.max(0, Math.min(plan.scenes.length - 1, to));
      history.replaceState(null, "", "#" + i);
      frame.show(i);
      if (i > 0) help.classList.add("gone");
    };
    wrap.addEventListener("click", function (e) { go(i + (e.clientX < window.innerWidth * 0.3 ? -1 : 1)); });
    window.addEventListener("keydown", function (e) {
      if (e.key === "ArrowRight" || e.key === " " || e.key === "Enter" || e.key === "PageDown") { e.preventDefault(); go(i + 1); }
      if (e.key === "ArrowLeft" || e.key === "Backspace" || e.key === "PageUp") { e.preventDefault(); go(i - 1); }
      if (e.key === "Home") go(0);
      if (e.key === "End") go(plan.scenes.length - 1);
    });
    window.addEventListener("resize", size);
    size();
    frame.ready.then(function () { go(i); });
    return frame;
  }

  /** @type {any} */ (root).PVC = { create: create, guide: guide, metrics: metrics, fontCss: fontCss, installFonts: installFonts };
})(typeof globalThis !== "undefined" ? globalThis : window);
