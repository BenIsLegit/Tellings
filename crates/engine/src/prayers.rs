//! The words. Traditional wording, copied line for line from the first video
//! (`prototype/core.js`). Do not reword anything here. A person who wants
//! different wording sets their own text in a setup; that never touches this file.

pub struct PrayerText {
    pub key: &'static str,
    pub title: &'static str,
    pub lines: &'static [&'static str],
    /// When two voices take turns, the second voice starts at this line.
    /// `None` means the prayer is said straight through by the leader.
    pub response_from: Option<usize>,
}

pub const PRAYERS: &[PrayerText] = &[
    PrayerText {
        key: "sign",
        title: "Sign of the Cross",
        lines: &[
            "In the name of the Father,",
            "and of the Son,",
            "and of the Holy Spirit.",
            "Amen.",
        ],
        response_from: None,
    },
    PrayerText {
        key: "creed",
        title: "Apostles' Creed",
        lines: &[
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
            "Amen.",
        ],
        response_from: None,
    },
    PrayerText {
        key: "ourfather",
        title: "Our Father",
        lines: &[
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
            "Amen.",
        ],
        response_from: Some(5),
    },
    PrayerText {
        key: "hailmary",
        title: "Hail Mary",
        lines: &[
            "Hail Mary, full of grace,",
            "the Lord is with thee.",
            "Blessed art thou among women,",
            "and blessed is the fruit of thy womb, Jesus.",
            "Holy Mary, Mother of God,",
            "pray for us sinners,",
            "now and at the hour of our death.",
            "Amen.",
        ],
        response_from: Some(4),
    },
    PrayerText {
        key: "glorybe",
        title: "Glory Be",
        lines: &[
            "Glory be to the Father,",
            "and to the Son,",
            "and to the Holy Spirit,",
            "as it was in the beginning,",
            "is now, and ever shall be,",
            "world without end.",
            "Amen.",
        ],
        response_from: Some(3),
    },
    PrayerText {
        key: "fatima",
        title: "O My Jesus",
        lines: &[
            "O my Jesus,",
            "forgive us our sins,",
            "save us from the fires of hell,",
            "and lead all souls to heaven,",
            "especially those in most need of thy mercy.",
            "Amen.",
        ],
        response_from: None,
    },
    PrayerText {
        key: "hhq",
        title: "Hail, Holy Queen",
        lines: &[
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
            "Amen.",
        ],
        response_from: None,
    },
    PrayerText {
        key: "closing",
        title: "Closing Prayer",
        lines: &[
            "O God, whose only begotten Son,",
            "by his life, death, and resurrection,",
            "has purchased for us the rewards of eternal life,",
            "grant, we beseech thee,",
            "that by meditating upon these mysteries",
            "of the most holy Rosary of the Blessed Virgin Mary,",
            "we may imitate what they contain",
            "and obtain what they promise,",
            "through the same Christ our Lord.",
            "Amen.",
        ],
        response_from: None,
    },
];

pub fn prayer(key: &str) -> Option<&'static PrayerText> {
    PRAYERS.iter().find(|p| p.key == key)
}

pub struct Mystery {
    pub name: &'static str,
    /// Where the scene is told in Scripture, if it is.
    pub reference: Option<&'static str>,
    pub note: Option<&'static str>,
}

pub struct MysterySet {
    pub key: &'static str,
    pub name: &'static str,
    /// Shown on the opening title: "Prayed on Thursdays".
    pub days: &'static str,
    pub items: [Mystery; 5],
}

const fn m(name: &'static str, reference: &'static str) -> Mystery {
    Mystery { name, reference: Some(reference), note: None }
}

pub const SETS: &[MysterySet] = &[
    MysterySet {
        key: "joyful",
        name: "Joyful",
        days: "Mondays and Saturdays",
        items: [
            m("The Annunciation", "Luke 1:26-38"),
            m("The Visitation", "Luke 1:39-56"),
            m("The Nativity", "Luke 2:1-20"),
            m("The Presentation in the Temple", "Luke 2:22-38"),
            m("The Finding of Jesus in the Temple", "Luke 2:41-52"),
        ],
    },
    MysterySet {
        key: "sorrowful",
        name: "Sorrowful",
        days: "Tuesdays and Fridays",
        items: [
            m("The Agony in the Garden", "Matthew 26:36-46"),
            m("The Scourging at the Pillar", "John 19:1"),
            m("The Crowning with Thorns", "Matthew 27:27-31"),
            m("The Carrying of the Cross", "Luke 23:26-32"),
            m("The Crucifixion", "Luke 23:33-46"),
        ],
    },
    MysterySet {
        key: "glorious",
        name: "Glorious",
        days: "Wednesdays and Sundays",
        items: [
            m("The Resurrection", "Matthew 28:1-10"),
            m("The Ascension", "Acts 1:6-11"),
            m("The Descent of the Holy Spirit", "Acts 2:1-13"),
            Mystery {
                name: "The Assumption of Mary",
                reference: None,
                note: Some("This one is not narrated in Scripture. It comes from Church tradition."),
            },
            Mystery {
                name: "The Coronation of Mary",
                reference: Some("Revelation 12:1"),
                note: Some("This one is not narrated directly in Scripture. It is often read with this verse."),
            },
        ],
    },
    MysterySet {
        key: "luminous",
        name: "Luminous",
        days: "Thursdays",
        items: [
            m("The Baptism in the Jordan", "Matthew 3:13-17"),
            m("The Wedding at Cana", "John 2:1-11"),
            m("The Proclamation of the Kingdom", "Mark 1:14-15"),
            m("The Transfiguration", "Matthew 17:1-8"),
            m("The Institution of the Eucharist", "Matthew 26:26-29"),
        ],
    },
];

pub fn set(key: &str) -> Option<&'static MysterySet> {
    SETS.iter().find(|s| s.key == key)
}

/// Sunday first. The usual custom for each day of the week.
pub const DAY_SET: [&str; 7] = ["glorious", "joyful", "sorrowful", "glorious", "luminous", "sorrowful", "joyful"];

pub const ROMAN: [&str; 5] = ["I", "II", "III", "IV", "V"];
pub const ORDINAL: [&str; 5] = ["first", "second", "third", "fourth", "fifth"];
/// The three opening Hail Marys are often offered for these.
pub const VIRTUES: [&str; 3] = ["faith", "hope", "love"];

/// Which set of mysteries is usual on a given date.
pub fn set_for_date(date: chrono::NaiveDate) -> &'static str {
    use chrono::Datelike;
    DAY_SET[date.weekday().num_days_from_sunday() as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn mysteries_follow_the_week() {
        // 2026-11-16 is a Monday.
        let monday = NaiveDate::from_ymd_opt(2026, 11, 16).unwrap();
        let expect = ["joyful", "sorrowful", "glorious", "luminous", "sorrowful", "joyful", "glorious"];
        for (i, want) in expect.iter().enumerate() {
            let day = monday + chrono::Duration::days(i as i64);
            assert_eq!(set_for_date(day), *want, "{day}");
        }
    }

    #[test]
    fn wording_matches_the_first_video() {
        // Spot checks against prototype/core.js. If one of these fails, a prayer was reworded.
        assert_eq!(prayer("hailmary").unwrap().lines[1], "the Lord is with thee.");
        assert_eq!(prayer("ourfather").unwrap().lines[0], "Our Father, who art in heaven,");
        assert_eq!(prayer("creed").unwrap().lines.len(), 19);
        assert_eq!(prayer("hhq").unwrap().lines.len(), 14);
        assert_eq!(prayer("closing").unwrap().lines[8], "through the same Christ our Lord.");
        assert_eq!(prayer("fatima").unwrap().lines[4], "especially those in most need of thy mercy.");
        assert_eq!(PRAYERS.len(), 8);
    }

    #[test]
    fn the_second_voice_starts_where_a_group_answers() {
        let at = |k: &str| prayer(k).unwrap().lines[prayer(k).unwrap().response_from.unwrap()];
        assert_eq!(at("ourfather"), "Give us this day our daily bread;");
        assert_eq!(at("hailmary"), "Holy Mary, Mother of God,");
        assert_eq!(at("glorybe"), "as it was in the beginning,");
    }
}
