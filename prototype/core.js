// Extracted from rosary-guide.html. Do not edit here.

const PRAYERS = {
  sign: { title: "Sign of the Cross", lines: [
    "In the name of the Father,",
    "and of the Son,",
    "and of the Holy Spirit.",
    "Amen."
  ] },
  creed: { title: "Apostles' Creed", lines: [
    "I believe in God, the Father almighty,",
    "Creator of heaven and earth,",
    "and in Jesus Christ, his only Son, our Lord,",
    "who was conceived by the Holy Spirit,",
    "born of the Virgin Mary,",
    "suffered under Pontius Pilate,",
    "was crucified, died and was buried;",
    "he descended into hell;",
    "on the third day he rose again from the dead;",
    "he ascended into heaven,",
    "and is seated at the right hand of God the Father almighty;",
    "from there he will come to judge the living and the dead.",
    "I believe in the Holy Spirit,",
    "the holy catholic Church,",
    "the communion of saints,",
    "the forgiveness of sins,",
    "the resurrection of the body,",
    "and life everlasting.",
    "Amen."
  ] },
  ourfather: { title: "Our Father", lines: [
    "Our Father, who art in heaven,",
    "hallowed be thy name;",
    "thy kingdom come;",
    "thy will be done",
    "on earth as it is in heaven.",
    "Give us this day our daily bread;",
    "and forgive us our trespasses,",
    "as we forgive those who trespass against us;",
    "and lead us not into temptation,",
    "but deliver us from evil.",
    "Amen."
  ] },
  hailmary: { title: "Hail Mary", lines: [
    "Hail Mary, full of grace,",
    "the Lord is with thee.",
    "Blessed art thou among women,",
    "and blessed is the fruit of thy womb, Jesus.",
    "Holy Mary, Mother of God,",
    "pray for us sinners,",
    "now and at the hour of our death.",
    "Amen."
  ] },
  glorybe: { title: "Glory Be", lines: [
    "Glory be to the Father,",
    "and to the Son,",
    "and to the Holy Spirit,",
    "as it was in the beginning,",
    "is now, and ever shall be,",
    "world without end.",
    "Amen."
  ] },
  fatima: { title: "O My Jesus", lines: [
    "O my Jesus,",
    "forgive us our sins,",
    "save us from the fires of hell,",
    "and lead all souls to heaven,",
    "especially those in most need of thy mercy.",
    "Amen."
  ] },
  hhq: { title: "Hail, Holy Queen", lines: [
    "Hail, holy Queen, Mother of mercy,",
    "our life, our sweetness, and our hope.",
    "To thee do we cry,",
    "poor banished children of Eve.",
    "To thee do we send up our sighs,",
    "mourning and weeping in this valley of tears.",
    "Turn then, most gracious advocate,",
    "thine eyes of mercy toward us,",
    "and after this, our exile,",
    "show unto us the blessed fruit of thy womb, Jesus.",
    "O clement, O loving, O sweet Virgin Mary.",
    "Pray for us, O holy Mother of God,",
    "that we may be made worthy of the promises of Christ.",
    "Amen."
  ] },
  closing: { title: "Closing Prayer", lines: [
    "O God, whose only begotten Son,",
    "by his life, death, and resurrection,",
    "has purchased for us the rewards of eternal life,",
    "grant, we beseech thee,",
    "that by meditating upon these mysteries",
    "of the most holy Rosary of the Blessed Virgin Mary,",
    "we may imitate what they contain",
    "and obtain what they promise,",
    "through the same Christ our Lord.",
    "Amen."
  ] }
};

const SETS = {
  joyful: { name: "Joyful", items: [
    { name: "The Annunciation", ref: "Luke 1:26-38" },
    { name: "The Visitation", ref: "Luke 1:39-56" },
    { name: "The Nativity", ref: "Luke 2:1-20" },
    { name: "The Presentation in the Temple", ref: "Luke 2:22-38" },
    { name: "The Finding of Jesus in the Temple", ref: "Luke 2:41-52" }
  ] },
  sorrowful: { name: "Sorrowful", items: [
    { name: "The Agony in the Garden", ref: "Matthew 26:36-46" },
    { name: "The Scourging at the Pillar", ref: "John 19:1" },
    { name: "The Crowning with Thorns", ref: "Matthew 27:27-31" },
    { name: "The Carrying of the Cross", ref: "Luke 23:26-32" },
    { name: "The Crucifixion", ref: "Luke 23:33-46" }
  ] },
  glorious: { name: "Glorious", items: [
    { name: "The Resurrection", ref: "Matthew 28:1-10" },
    { name: "The Ascension", ref: "Acts 1:6-11" },
    { name: "The Descent of the Holy Spirit", ref: "Acts 2:1-13" },
    { name: "The Assumption of Mary", ref: null, note: "This one is not narrated in Scripture. It comes from Church tradition." },
    { name: "The Coronation of Mary", ref: "Revelation 12:1", note: "This one is not narrated directly in Scripture. It is often read with this verse." }
  ] },
  luminous: { name: "Luminous", items: [
    { name: "The Baptism in the Jordan", ref: "Matthew 3:13-17" },
    { name: "The Wedding at Cana", ref: "John 2:1-11" },
    { name: "The Proclamation of the Kingdom", ref: "Mark 1:14-15" },
    { name: "The Transfiguration", ref: "Matthew 17:1-8" },
    { name: "The Institution of the Eucharist", ref: "Matthew 26:26-29" }
  ] }
};

// Sunday first. The usual custom for each day of the week.
const DAY_SET = ["glorious", "joyful", "sorrowful", "glorious", "luminous", "sorrowful", "joyful"];
const DAY_NAME = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
const ROMAN = ["I", "II", "III", "IV", "V"];
const ORDINAL = ["first", "second", "third", "fourth", "fifth"];

function buildSteps() {
  const s = [];
  s.push({ bead: "cross", prayer: "sign", where: "Hold the crucifix", phase: "open" });
  s.push({ bead: "cross", prayer: "creed", where: "Still holding the crucifix", phase: "open" });
  s.push({ bead: "p1", prayer: "ourfather", where: "First big bead", phase: "open" });
  ["faith", "hope", "love"].forEach(function (virtue, i) {
    s.push({ bead: "p" + (i + 2), prayer: "hailmary", where: "Small bead " + (i + 1) + " of 3", pip: [i + 1, 3], hint: "Many people offer this one for " + virtue + ".", phase: "open" });
  });
  s.push({ bead: "g0", prayer: "glorybe", where: "No bead for this one. Hold the chain before the next big bead.", phase: "open" });
  for (let d = 1; d <= 5; d++) {
    s.push({ bead: d === 1 ? "p5" : "L" + d, prayer: "ourfather", where: d === 1 ? "Second big bead, just below the medal" : "Big bead", decade: d, announce: true, phase: "decade" });
    for (let b = 1; b <= 10; b++) {
      s.push({ bead: "d" + d + "b" + b, prayer: "hailmary", where: "Small bead " + b + " of 10", pip: [b, 10], decade: d, phase: "decade" });
    }
    s.push({ bead: "g" + d, prayer: "glorybe", where: "No bead for this one. Hold the chain after the tenth small bead.", decade: d, phase: "decade" });
    s.push({ bead: "g" + d, prayer: "fatima", where: "Still on the chain", optional: "Optional. Many people add this after each Glory Be.", decade: d, phase: "decade" });
  }
  s.push({ bead: "medal", prayer: "hhq", where: "You are back at the medal", phase: "close" });
  s.push({ bead: "medal", prayer: "closing", where: "Still on the medal", optional: "Optional.", phase: "close" });
  s.push({ bead: "cross", prayer: "sign", where: "Finish on the crucifix", phase: "close" });
  return s;
}

// Bead positions. Loop centre (180,160), radius 135, medal at the bottom,
// decades run up the right-hand side first. Big beads and the medal get extra chain.
function layout() {
  const C = { x: 180, y: 160 }, R = 135;
  const U_SMALL = 1, U_BIG = 1.8, U_MEDAL = 2.2;
  const total = 2 * U_MEDAL + 45 * U_SMALL + 8 * U_BIG;
  const deg = 360 / total;
  const at = function (u) {
    const a = (90 - u * deg) * Math.PI / 180;
    return { x: Math.round((C.x + R * Math.cos(a)) * 10) / 10, y: Math.round((C.y + R * Math.sin(a)) * 10) / 10 };
  };
  const P = {};
  P.cross = { x: 180, y: 445, kind: "cross" };
  P.p1 = { x: 180, y: 401, kind: "big" };
  P.p2 = { x: 180, y: 382, kind: "small" };
  P.p3 = { x: 180, y: 368, kind: "small" };
  P.p4 = { x: 180, y: 354, kind: "small" };
  P.g0 = { x: 180, y: 340.5, kind: "gap" };
  P.p5 = { x: 180, y: 325, kind: "big" };
  P.medal = Object.assign(at(0), { kind: "medal" });
  let u = U_MEDAL;
  for (let d = 1; d <= 5; d++) {
    if (d > 1) {
      P["L" + d] = Object.assign(at(u), { kind: "big" });
      u += U_BIG;
    }
    for (let b = 1; b <= 10; b++) {
      P["d" + d + "b" + b] = Object.assign(at(u), { kind: "small" });
      if (b < 10) u += U_SMALL;
    }
    const gapLen = d < 5 ? U_BIG : U_MEDAL;
    P["g" + d] = Object.assign(at(u + gapLen * (d < 5 ? 0.45 : 0.36)), { kind: "gap" });
    u += gapLen;
  }
  return { P: P, total: total, end: u };
}
