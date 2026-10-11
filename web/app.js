// The page draws what the core returns and decides nothing. Every call goes
// through the shim, which returns {"ok": ...} or {"error": "..."}. The game
// state is the JSON the core last returned; the page hands it back unread.
//
// Presentation state kept here, and why it is not a rule: which listing's
// sheet is open (`openKey`) and what the player has typed in an offer box
// (`drafts`). Neither changes what the core decides.
import init, { start, act, hint } from "./pkg/lowball_shim.js";

const $ = (id) => document.getElementById(id);
let content = "";
let state = "";
let view = null;
let openKey = null;
let drafts = {};

function el(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text !== undefined) e.textContent = text;
  return e;
}

function seed() {
  const a = new Uint32Array(1);
  crypto.getRandomValues(a);
  return String(a[0]);
}

function say(text) {
  $("message").textContent = text;
}

// Takes a core result; returns the error text, or "" when the step was taken.
function take(raw) {
  const result = JSON.parse(raw);
  if (result.error) return result.error;
  const was = view && view.phase;
  state = JSON.stringify(result.ok.state);
  view = result.ok.view;
  if (was !== view.phase) {
    drafts = {};
    window.scrollTo(0, 0);
  }
  say("");
  render();
  return "";
}

function send(action) {
  return take(act(content, state, JSON.stringify(action)));
}

function begin() {
  openKey = null;
  const problem = take(start(content, seed()));
  if (problem) say(problem);
}

// ---- drawing ---------------------------------------------------------------

function render() {
  document.body.dataset.phase = view.phase;
  const top = view.top;
  $("brand").textContent = top.brand;
  $("brand-tag").textContent = top.brand_tag;
  $("screen").textContent = top.screen;
  $("saved-label").textContent = top.saved_label;
  $("saved").textContent = top.saved;
  $("title-screen").hidden = !view.title;
  $("browse").hidden = !view.browse;
  $("summary").hidden = !view.summary;
  $("final").hidden = !view.final;
  if (view.title) drawTitle(view.title);
  if (view.browse) drawBrowse(view.browse);
  if (view.summary) drawSummary(view.summary);
  if (view.final) drawFinal(view.final);
  const sheet = $("detail");
  if (!view.browse && sheet.open) sheet.close();
}

function drawTitle(t) {
  $("title-art").alt = t.art_alt;
  $("title-name").textContent = t.name;
  $("title-tagline").textContent = t.tagline;
  $("title-how").replaceChildren(...t.how.map((line) => el("li", "", line)));
  $("start").textContent = t.button;
}

function meterEl(m) {
  const box = el("div", "meter");
  box.append(el("span", "meter-label", m.label));
  const pips = el("span", "pips");
  pips.setAttribute("aria-hidden", "true");
  m.pips.forEach((on) => pips.append(el("span", on ? "pip on" : "pip")));
  box.append(pips, el("span", "meter-count", m.count));
  return box;
}

function badgesEl(badges) {
  const box = el("span", "badges");
  badges.forEach((b) => box.append(el("span", "badge badge-" + b.kind, b.text)));
  return box;
}

function cardEl(c) {
  const card = el("button", c.done ? "card done" : "card");
  card.type = "button";
  card.dataset.key = c.key;
  const photo = el("span", "card-photo");
  const img = el("img");
  img.src = c.art;
  img.alt = c.alt;
  photo.append(img, badgesEl(c.badges));
  const body = el("span", "card-body");
  body.append(el("span", "card-price", c.price), el("span", "card-title", c.title), el("span", "card-meta", c.meta));
  card.append(photo, body);
  card.addEventListener("click", () => openSheet(c.key));
  return card;
}

function drawBrowse(b) {
  $("browse-heading").textContent = b.heading;
  $("browse-how").textContent = b.how;
  $("meters").replaceChildren(...b.meters.map(meterEl));
  $("grid").replaceChildren(...b.cards.map(cardEl));
  if (openKey !== null) {
    const c = b.cards.find((x) => x.key === openKey);
    if (c) drawSheet(c);
  }
}

function openSheet(key) {
  openKey = key;
  drawBrowse(view.browse);
}

function actionButton(b, kind, key) {
  const btn = el("button", "secondary", b.label);
  btn.type = "button";
  btn.disabled = !b.enabled;
  btn.addEventListener("click", () => {
    const problem = send({ kind: kind, listing: key });
    if (problem) $("sheet-error").textContent = problem;
    const intel = document.querySelector("#detail .intel");
    if (intel) intel.scrollIntoView({ block: "nearest" });
  });
  return btn;
}

function profileEl(p) {
  const box = el("section", "intel profile");
  const facts = el("ul", "facts");
  p.facts.forEach((f) => facts.append(el("li", "", f)));
  box.append(facts, el("p", "responds", p.responds), el("p", "bio", p.bio));
  box.append(el("h4", "", p.reviews_head));
  const reviews = el("ul", "reviews");
  p.reviews.forEach((r) => {
    const li = el("li");
    li.append(el("span", "stars", r.stars), el("span", "", r.text));
    reviews.append(li);
  });
  box.append(reviews, el("h4", "", p.sales_head));
  const sales = el("ul", "sales");
  p.sales.forEach((s) => {
    const li = el("li");
    li.append(el("b", "", s.title), el("span", "", s.line));
    sales.append(li);
  });
  box.append(sales);
  return box;
}

function chatEl(c, seller) {
  const box = el("section", "intel chat");
  const mine = el("div", "bubble mine");
  mine.append(el("span", "who", c.you), el("span", "", c.question));
  const theirs = el("div", "bubble theirs");
  theirs.append(el("span", "who", seller), el("span", "", c.reply));
  box.append(mine, theirs);
  return box;
}

function offerEl(c) {
  const o = c.offer;
  if (!o.open) return el("p", "sent", o.sent);
  const form = el("form", "offer");
  const label = el("label", "offer-label", o.label);
  label.htmlFor = "offer-input";
  const row = el("div", "offer-row");
  const money = el("span", "currency", o.currency);
  const input = el("input");
  input.id = "offer-input";
  input.type = "text";
  input.inputMode = "numeric";
  input.autocomplete = "off";
  input.value = drafts[c.key] || o.start;
  const go = el("button", "primary", o.button);
  go.type = "submit";
  row.append(money, input, go);
  const range = el("input");
  range.id = "offer-range";
  range.type = "range";
  range.min = o.min;
  range.max = o.max;
  range.step = o.step;
  range.value = input.value;
  range.setAttribute("aria-label", o.label);
  const note = el("p", "offer-hint", o.hint);
  note.id = "offer-hint";
  const update = () => {
    drafts[c.key] = input.value;
    const r = JSON.parse(hint(content, state, c.key, input.value));
    note.textContent = r.error || r.ok;
  };
  input.addEventListener("input", () => {
    range.value = input.value;
    update();
  });
  range.addEventListener("input", () => {
    input.value = range.value;
    update();
  });
  form.addEventListener("submit", (e) => {
    e.preventDefault();
    const problem = send({ kind: "offer", listing: c.key, amount: input.value });
    if (problem) $("sheet-error").textContent = problem;
    else $("detail").close();
  });
  form.append(label, row, range, note);
  update();
  return form;
}

function drawSheet(c) {
  const sheet = $("detail");
  const box = el("div", "sheet");
  const close = el("button", "close", c.close);
  close.type = "button";
  close.addEventListener("click", () => sheet.close());
  const photo = el("div", "sheet-photo");
  const img = el("img");
  img.src = c.art;
  img.alt = c.alt;
  photo.append(img, badgesEl(c.badges));
  const info = el("div", "sheet-info");
  info.append(el("p", "sheet-price", c.price), el("h2", "", c.title), el("p", "sheet-meta", c.meta));
  info.append(el("h3", "", c.description_head), el("p", "desc", c.description));
  info.append(el("h3", "", c.seller_head));
  const who = el("div", "seller");
  who.append(el("span", "avatar avatar-" + c.colour, c.initial), el("span", "seller-name", c.seller));
  info.append(who);
  const actions = el("div", "actions");
  actions.append(actionButton(c.peek, "peek", c.key), actionButton(c.message, "message", c.key));
  info.append(actions);
  [c.peek.note, c.message.note].filter((t, k, all) => t && all.indexOf(t) === k).forEach((t) => info.append(el("p", "note", t)));
  if (c.profile) info.append(profileEl(c.profile));
  if (c.chat) info.append(chatEl(c.chat, c.seller));
  info.append(offerEl(c));
  const error = el("p", "sheet-error");
  error.id = "sheet-error";
  error.setAttribute("role", "status");
  info.append(error);
  box.append(close, photo, info);
  const scroll = sheet.scrollTop;
  sheet.replaceChildren(box);
  if (!sheet.open) sheet.showModal();
  else sheet.scrollTop = scroll;
}

function drawSummary(s) {
  $("summary-title").textContent = s.title;
  $("rows").replaceChildren(
    ...s.rows.map((r) => {
      const li = el("li", "row row-" + r.status);
      const img = el("img");
      img.src = r.art;
      img.alt = r.alt;
      const text = el("div", "row-text");
      const head = el("div", "row-head");
      head.append(el("b", "", r.title), el("span", "row-asking", r.asking));
      text.append(head, el("p", "", r.line));
      li.append(img, text, el("span", "pill pill-" + r.status, r.label));
      return li;
    })
  );
  $("total-label").textContent = s.total_label;
  $("total").textContent = s.total;
  $("next").textContent = s.button;
}

function statsEl(target, stats) {
  target.replaceChildren(
    ...stats.map((s) => {
      const d = el("div");
      d.append(el("dt", "", s.label), el("dd", "", s.value));
      return d;
    })
  );
}

function drawFinal(f) {
  $("final-art").alt = f.art_alt;
  $("final-eyebrow").textContent = f.eyebrow;
  $("final-pct").textContent = f.pct;
  $("final-line").textContent = f.line;
  $("final-rank").textContent = f.rank;
  $("final-rank-line").textContent = f.rank_line;
  statsEl($("final-stats"), f.stats);
  statsEl($("final-rounds"), f.rounds);
  $("again").textContent = f.button;
}

// ---- wiring ----------------------------------------------------------------

$("start").addEventListener("click", () => send({ kind: "start" }));
$("next").addEventListener("click", () => send({ kind: "next" }));
$("again").addEventListener("click", () => {
  begin();
  send({ kind: "start" });
});
$("detail").addEventListener("close", () => {
  openKey = null;
});
$("detail").addEventListener("click", (e) => {
  if (e.target === $("detail")) $("detail").close();
});

async function boot() {
  await init();
  const read = (name) => fetch("data/" + name + ".json").then((r) => r.text());
  const [items, sellers, copy] = await Promise.all([read("items"), read("sellers"), read("copy")]);
  content = "{\"items\":" + items + ",\"sellers\":" + sellers + ",\"copy\":" + copy + "}";
  begin();
  document.body.dataset.ready = "true";
}

boot();
