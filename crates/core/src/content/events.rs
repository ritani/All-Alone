//! Choice events: short situations with two to four options. Some chain
//! across days by setting flags and scheduling a follow-up.

use super::{Creature, Skill, Station, VitalKind, Weather};

/// Where an event can fire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ctx {
    /// When you rise at the lock house.
    Dawn,
    /// Coming home from an expedition, in daylight.
    Home,
    /// When you go to bed.
    Night,
    /// On the walk out.
    Road,
    /// On arriving at a place.
    Arrive,
    /// On first entering a specific room.
    Room,
    /// While listening to the radio.
    Radio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DogIs {
    NeverMet,
    Following,
    Companion,
}

#[derive(Clone, Copy, Debug)]
pub enum Cond {
    MinDay(u32),
    Flag(&'static str),
    NoFlag(&'static str),
    Item(&'static str, u32),
    /// Any food worth a meal (in the pack when out, the store at home).
    Meals(u32),
    /// Any water.
    Water(u32),
    Weapon,
    /// A firearm with at least one round.
    Gun,
    Light,
    Dog(DogIs),
    Weather(Weather),
    /// Dark outside (after dusk or before dawn).
    Dark,
    Station(Station, u8),
    StationBelow(Station, u8),
    Codex(&'static str),
    VitalAtLeast(VitalKind, i32),
    Trades(u32),
    NotFrozen,
}

pub use super::Band;

#[derive(Clone, Copy, Debug)]
pub enum DogFx {
    /// The dog has been seen and fed once; it may come back.
    Follow,
    /// The dog stays.
    Join,
    /// The dog is gone.
    Lose,
    Hurt(i32),
    Heal(i32),
}

#[derive(Clone, Copy, Debug)]
pub enum Fx {
    Item(&'static str, i32),
    /// Remove this many meals, least valuable first.
    Meals(u32),
    /// Remove this much water, dirty first.
    Water(u32),
    Vital(VitalKind, i32),
    Noise(i32),
    Flag(&'static str),
    Unflag(&'static str),
    /// Schedule a chained event this many days ahead (0 = later today).
    Schedule(&'static str, u32),
    Reveal(&'static str),
    Codex(&'static str),
    Fight(Creature, u8, Band),
    Dog(DogFx),
    Bleed(u8),
    Infect(i32),
    Xp(Skill, u32),
    Barricade(i32),
    Minutes(u32),
    /// Roll each (item, percent, min, max).
    Loot(&'static [(&'static str, i32, u32, u32)]),
    /// Add an item to what the widow will offer.
    WidowStock(&'static str),
    /// Use one round from a firearm (counts as a shot fired).
    Shot,
    /// Counts as a kill (a Stilled put down in the event).
    Kill,
    /// Lose up to this many units of the most valuable things in the store.
    Rob(u32),
}

#[derive(Clone, Copy, Debug)]
pub enum Odds {
    Always,
    Pct(i32),
    /// base + per x skill level.
    Skill(Skill, i32, i32),
    /// base + (nerve - 50) / 2.
    Nerve(i32),
}

#[derive(Clone, Copy, Debug)]
pub struct Res {
    pub text: &'static str,
    pub fx: &'static [Fx],
}

#[derive(Clone, Copy, Debug)]
pub struct Opt {
    pub label: &'static str,
    pub req: &'static [Cond],
    pub minutes: u32,
    pub odds: Odds,
    pub ok: Res,
    pub fail: Res,
}

#[derive(Clone, Copy, Debug)]
pub struct EventDef {
    pub id: &'static str,
    pub ctx: Ctx,
    /// Relative weight among eligible events; 0 = only when scheduled.
    pub weight: u32,
    pub once: bool,
    pub req: &'static [Cond],
    pub place: Option<&'static str>,
    pub room: Option<&'static str>,
    pub title: &'static str,
    pub text: &'static str,
    pub opts: &'static [Opt],
}

const NONE: Res = Res { text: "", fx: &[] };

const fn opt(label: &'static str, text: &'static str, fx: &'static [Fx]) -> Opt {
    Opt {
        label,
        req: &[],
        minutes: 0,
        odds: Odds::Always,
        ok: Res { text, fx },
        fail: NONE,
    }
}

/// An option body whose label is set by the caller.
const fn then(text: &'static str, fx: &'static [Fx]) -> Opt {
    opt("", text, fx)
}

const BLANK: Opt = opt("", "", &[]);

const E: EventDef = EventDef {
    id: "",
    ctx: Ctx::Dawn,
    weight: 10,
    once: true,
    req: &[],
    place: None,
    room: None,
    title: "",
    text: "",
    opts: &[],
};

use Fx::*;
use VitalKind::*;

pub static EVENTS: &[EventDef] = &[
    // ---------------------------------------------------------------- Dawn
    EventDef {
        id: "dog_at_gate",
        weight: 40,
        req: &[Cond::MinDay(2), Cond::Dog(DogIs::NeverMet)],
        title: "A dog at the gate",
        text: "A dog sits outside the gate, thin, rope-collared, watching the door. It does not bark. It has not barked, you realise, all the time it has been there.",
        opts: &[
            Opt {
                label: "Feed it",
                req: &[Cond::Meals(1)],
                ..then(
                    "You put food down by the gate and step back. It eats without taking its eyes off you, then lies down by the woodpile.",
                    &[
                        Meals(1),
                        Dog(DogFx::Follow),
                        Schedule("dog_returns", 1),
                        Vital(Nerve, 5),
                    ],
                )
            },
            opt(
                "Leave it",
                "You close the door. When you look again it has gone.",
                &[Schedule("dog_again", 3)],
            ),
            opt(
                "Throw a stone",
                "The stone cracks on the road. The dog flinches and goes, low and fast, into the reeds.",
                &[Noise(4), Flag("dog_driven")],
            ),
        ],
        ..E
    },
    EventDef {
        id: "dog_again",
        weight: 0,
        req: &[Cond::Dog(DogIs::NeverMet), Cond::NoFlag("dog_driven")],
        title: "The dog, again",
        text: "The rope-collared dog is back, lying at the gate with its head on its paws. Thinner.",
        opts: &[
            Opt {
                label: "Feed it",
                req: &[Cond::Meals(1)],
                ..then(
                    "It eats, and this time it lets you near.",
                    &[Meals(1), Dog(DogFx::Follow), Schedule("dog_returns", 1)],
                )
            },
            opt(
                "Leave it",
                "You leave it. It does not come again.",
                &[Flag("dog_driven")],
            ),
        ],
        ..E
    },
    EventDef {
        id: "dog_returns",
        weight: 0,
        req: &[Cond::Dog(DogIs::Following)],
        title: "The dog stays",
        text: "The dog is still by the woodpile. It stands when you open the door and waits to see what you will do.",
        opts: &[
            Opt {
                label: "Feed it and let it in",
                req: &[Cond::Meals(1)],
                ..then(
                    "It eats, then walks past you into the kitchen and lies down by the stove as if it has always slept there.",
                    &[Meals(1), Dog(DogFx::Join), Vital(Nerve, 10)],
                )
            },
            opt(
                "Send it off",
                "You shout and wave your arms. It goes, and does not look back.",
                &[Dog(DogFx::Lose), Noise(3)],
            ),
        ],
        ..E
    },
    EventDef {
        id: "tracks_mud",
        weight: 12,
        req: &[Cond::MinDay(3)],
        title: "Footprints",
        text: "Bare footprints in the mud around the house. They go twice round, close to the walls, and away toward the river.",
        opts: &[
            Opt {
                label: "Follow them",
                minutes: 45,
                odds: Odds::Pct(55),
                ..Opt {
                    ok: Res {
                        text: "They end at the water's edge, where someone has dropped a bag and walked straight in. You take the bag.",
                        fx: &[
                            Loot(&[
                                ("tinned_meat", 70, 1, 2),
                                ("bandage", 40, 1, 1),
                                ("candle", 50, 1, 2),
                                ("rounds", 20, 1, 3),
                            ]),
                            Vital(Nerve, -5),
                            Codex("fg_water"),
                        ],
                    },
                    fail: Res {
                        text: "They lead into the reeds and you lose them. When you turn back the house looks very small.",
                        fx: &[Vital(Nerve, -6)],
                    },
                    ..BLANK
                }
            },
            opt(
                "Bar the door and stay in",
                "You check every board twice. It costs you the morning and some nerve.",
                &[Minutes(60), Vital(Nerve, -4), Barricade(5)],
            ),
            opt(
                "Scuff them out",
                "You scrape them away with your boot. It does not help.",
                &[Vital(Nerve, -2)],
            ),
        ],
        ..E
    },
    EventDef {
        id: "smoke_downriver",
        weight: 10,
        req: &[Cond::MinDay(4), Cond::NoFlag("signal_sent")],
        title: "Smoke",
        text: "A thin column of smoke downriver, past the trees. Someone has a fire.",
        opts: &[
            opt(
                "Watch it a while",
                "It burns for an hour and goes out. Someone else is alive. That is something.",
                &[Minutes(60), Vital(Nerve, 8)],
            ),
            opt(
                "Answer with your own smoke",
                "You build up the stove with wet wood until it smokes black. Anyone could see it. Anything could hear the fire.",
                &[
                    Item("firewood", -1),
                    Noise(12),
                    Flag("signal_sent"),
                    Schedule("men_at_gate", 2),
                ],
            ),
            opt("Ignore it", "You go about the day.", &[]),
        ],
        ..E
    },
    EventDef {
        id: "rats",
        weight: 10,
        req: &[Cond::MinDay(4), Cond::Meals(4), Cond::StationBelow(Station::Store, 2)],
        title: "Rats",
        text: "Something has been in the store. Paper chewed, a tin rolled under the shelf, droppings.",
        opts: &[
            Opt {
                label: "Set a snare for them",
                req: &[Cond::Item("snare", 1)],
                ..then("By noon the snare has one. The rest stay away.", &[Minutes(30)])
            },
            opt(
                "Clear the store out",
                "You take everything off the shelves, sweep, and put it all back. Some of it was already spoiled.",
                &[Minutes(60), Meals(1)],
            ),
            opt(
                "Leave it",
                "They come back that night, and the night after.",
                &[Meals(3)],
            ),
        ],
        ..E
    },
    EventDef {
        id: "cold_snap",
        weight: 14,
        req: &[Cond::MinDay(18)],
        title: "Cold snap",
        text: "The cold came in overnight. The water in the jug is solid and the window is furred white on the inside.",
        opts: &[
            Opt {
                label: "Burn extra wood",
                req: &[Cond::Item("firewood", 2)],
                ..then(
                    "You feed the stove until the iron ticks. Warmth comes back into your hands.",
                    &[Item("firewood", -2), Vital(Warmth, 25)],
                )
            },
            opt(
                "Spend the morning under blankets",
                "You lie still until the sun is up. Warmer, and the morning gone.",
                &[Minutes(150), Vital(Warmth, 15), Vital(Rest, 10)],
            ),
            opt(
                "Get up and get on",
                "You move to keep warm. It nearly works.",
                &[Vital(Warmth, -8)],
            ),
        ],
        ..E
    },
    EventDef {
        id: "barrel_ice",
        weight: 10,
        req: &[Cond::MinDay(22), Cond::Station(Station::Barrel, 1)],
        title: "Ice in the barrel",
        text: "The rain barrel has a lid of ice two fingers thick. The water under it will freeze through by tonight.",
        opts: &[
            opt(
                "Break it and bail",
                "You break the ice with the hammer and bail what you can into the kitchen. Your hands burn with it.",
                &[Minutes(40), Item("water_dirty", 3), Vital(Warmth, -10)],
            ),
            opt("Leave it", "By night the barrel is a block.", &[]),
        ],
        ..E
    },
    EventDef {
        id: "dog_limping",
        weight: 10,
        req: &[Cond::MinDay(8), Cond::Dog(DogIs::Companion)],
        title: "The dog is limping",
        text: "The dog holds a forepaw off the floor. There is a cut on the pad, deep, and it licks at it.",
        opts: &[
            Opt {
                label: "Bind it",
                req: &[Cond::Item("bandage", 1)],
                ..then(
                    "You clean it and bind it. The dog lets you, which surprises you both.",
                    &[Item("bandage", -1), Dog(DogFx::Heal(30)), Xp(Skill::FieldMedicine, 1)],
                )
            },
            Opt {
                label: "Bind it with rags",
                req: &[Cond::Item("rag", 1)],
                ..then(
                    "Rags will have to do. It worries at them all morning.",
                    &[Item("rag", -1), Dog(DogFx::Heal(15))],
                )
            },
            opt("Leave it", "It limps after you all day.", &[Dog(DogFx::Hurt(25))]),
        ],
        ..E
    },
    EventDef {
        id: "snow_tracks",
        weight: 16,
        req: &[Cond::Weather(Weather::Snow)],
        title: "Tracks in the snow",
        text: "New snow, and in it a line of tracks that circles the house twice and stops below the window where you sleep.",
        opts: &[
            Opt {
                label: "Reinforce the window",
                req: &[Cond::Item("plank", 1), Cond::Item("nails", 2)],
                ..then(
                    "You board the window from inside. The room is darker now.",
                    &[
                        Item("plank", -1),
                        Item("nails", -2),
                        Barricade(20),
                        Minutes(40),
                        Noise(5),
                    ],
                )
            },
            opt(
                "Follow them",
                "They go down to the river and end at the ice's edge. Nothing comes back out.",
                &[Minutes(40), Vital(Nerve, -8), Codex("fg_water")],
            ),
            opt(
                "Ignore them",
                "You do not look out of that window all day.",
                &[Vital(Nerve, -5)],
            ),
        ],
        ..E
    },
    EventDef {
        id: "parcel_step",
        weight: 0,
        req: &[],
        title: "On the step",
        text: "On the doorstep in the morning: a jar of water, stoppered, and a photograph weighted with a stone.",
        opts: &[opt(
            "Take them in",
            "Someone was grateful.",
            &[Item("water_clean", 1), Codex("photo_hale"), Vital(Nerve, 5)],
        )],
        ..E
    },
    EventDef {
        id: "ivy_gone",
        weight: 0,
        req: &[],
        title: "The woman has gone",
        text: "The blankets by the stove are folded. She has gone before light.",
        opts: &[Opt {
            label: "Look around",
            odds: Odds::Pct(70),
            ..Opt {
                ok: Res {
                    text: "She has left a note by the stove, and a roll of bandages on top of it.",
                    fx: &[Codex("note_ivy"), Item("bandage", 2), Item("antiseptic", 1)],
                },
                fail: Res {
                    text: "The store is lighter than it was. She left nothing.",
                    fx: &[Meals(2), Vital(Nerve, -6)],
                },
                ..BLANK
            }
        }],
        ..E
    },
    // ---------------------------------------------------------------- Home (back from a trip)
    EventDef {
        id: "men_at_gate",
        ctx: Ctx::Home,
        weight: 0,
        title: "Men at the gate",
        text: "Two men with a handcart at the gate. Thin, armed with a fowling piece and a spade. They saw the smoke. They want food.",
        opts: &[
            Opt {
                label: "Give them food",
                req: &[Cond::Meals(2)],
                ..then(
                    "You pass out two meals. The older one nods and leaves a box on the post: shells and a little lamp oil. They go downriver.",
                    &[
                        Meals(2),
                        Item("shells", 2),
                        Item("lamp_oil", 1),
                        Flag("men_friendly"),
                        Codex("note_stranger"),
                    ],
                )
            },
            Opt {
                label: "Refuse through the door",
                odds: Odds::Pct(50),
                ..Opt {
                    ok: Res {
                        text: "They argue between themselves, then go.",
                        fx: &[Vital(Nerve, -5)],
                    },
                    fail: Res {
                        text: "They go at the door with the spade. The boards hold, mostly. The noise goes on a long time.",
                        fx: &[Barricade(-30), Noise(20), Vital(Nerve, -10)],
                    },
                    ..BLANK
                }
            },
            Opt {
                label: "Show them the gun",
                req: &[Cond::Gun],
                ..then(
                    "They see it and go, walking backwards for a while.",
                    &[Vital(Nerve, -4)],
                )
            },
            Opt {
                label: "Let them in",
                odds: Odds::Pct(55),
                ..Opt {
                    ok: Res {
                        text: "They eat with you and talk about the estuary. One of them draws you a map before they go.",
                        fx: &[Meals(1), Codex("note_stranger"), Vital(Nerve, 12)],
                    },
                    fail: Res {
                        text: "One of them holds the spade while the other fills the cart from your store.",
                        fx: &[Rob(6), Vital(Nerve, -15)],
                    },
                    ..BLANK
                }
            },
        ],
        ..E
    },
    EventDef {
        id: "visitor_ferry",
        ctx: Ctx::Home,
        weight: 8,
        req: &[Cond::MinDay(6)],
        title: "Someone at the slip",
        text: "There is someone on the far slip, beside the ferry, looking at it. A man, alive, with a sack.",
        opts: &[
            opt(
                "Call across the water",
                "He looks up, startled, and runs. He has left the sack.",
                &[
                    Noise(6),
                    Loot(&[("tinned_beans", 80, 1, 2), ("wire", 60, 1, 2), ("parts", 40, 1, 1)]),
                ],
            ),
            opt(
                "Watch him",
                "He walks round the ferry twice, shakes his head, and leaves. He thinks it is beyond saving.",
                &[Vital(Nerve, 3)],
            ),
        ],
        ..E
    },
    EventDef {
        id: "house_tampered",
        ctx: Ctx::Home,
        weight: 8,
        req: &[Cond::MinDay(5)],
        title: "Scratches on the door",
        text: "Long scratches down the boards of the door, at the height of a hand, and the latch bent.",
        opts: &[
            Opt {
                label: "Mend the door",
                req: &[Cond::Item("nails", 2)],
                ..then(
                    "You straighten the latch and nail a batten across.",
                    &[Item("nails", -2), Barricade(12), Minutes(30), Noise(4)],
                )
            },
            opt("Leave it", "You go in. The scratches stay.", &[Vital(Nerve, -4)]),
        ],
        ..E
    },
    // ---------------------------------------------------------------- Night
    EventDef {
        id: "knock",
        ctx: Ctx::Night,
        weight: 14,
        req: &[Cond::MinDay(3)],
        title: "Three knocks",
        text: "Three knocks on the door. Even, unhurried. Then nothing.",
        opts: &[
            Opt {
                label: "Call out",
                odds: Odds::Pct(50),
                ..Opt {
                    ok: Res {
                        text: "A woman's voice, hoarse: please, water. Only water.",
                        fx: &[Noise(4), Schedule("woman_at_door", 0)],
                    },
                    fail: Res {
                        text: "Your voice sounds very loud. Nothing answers. You do not sleep for a long time.",
                        fx: &[Noise(6), Vital(Nerve, -10), Vital(Rest, -10)],
                    },
                    ..BLANK
                }
            },
            opt(
                "Keep still",
                "You hold your breath until your chest hurts. Nothing more comes.",
                &[Vital(Nerve, -8)],
            ),
            Opt {
                label: "Open the door",
                odds: Odds::Pct(45),
                ..Opt {
                    ok: Res {
                        text: "Nobody there. On the step, a paper bag of tins, and wet footprints going away.",
                        fx: &[Loot(&[("tinned_meat", 80, 1, 2), ("tinned_peaches", 50, 1, 1)])],
                    },
                    fail: Res {
                        text: "It is standing on the step, its knuckles still raised.",
                        fx: &[Fight(Creature::Stander, 1, Band::Close)],
                    },
                    ..BLANK
                }
            },
        ],
        ..E
    },
    EventDef {
        id: "woman_at_door",
        ctx: Ctx::Night,
        weight: 0,
        title: "The woman at the door",
        text: "Through the crack of the door: a woman, soaked to the waist, a nurse's cape over her coat. She asks for water and to sleep by your stove. She says she will be gone by morning.",
        opts: &[
            Opt {
                label: "Let her in",
                req: &[Cond::Water(2)],
                ..then(
                    "She drinks, sits by the stove and sleeps sitting up. You watch her for a long time.",
                    &[Water(2), Vital(Nerve, 6), Schedule("ivy_gone", 1)],
                )
            },
            Opt {
                label: "Pass water through the door",
                req: &[Cond::Water(1)],
                ..then(
                    "She drinks it in the doorway and thanks you, and goes. You bar the door again.",
                    &[Water(1), Vital(Nerve, 4), Schedule("parcel_step", 1)],
                )
            },
            opt(
                "Turn her away",
                "She goes without arguing. That is worse.",
                &[Vital(Nerve, -12)],
            ),
        ],
        ..E
    },
    EventDef {
        id: "roof",
        ctx: Ctx::Night,
        weight: 12,
        req: &[Cond::MinDay(2)],
        title: "On the roof",
        text: "Something moves on the roof. A slow shifting of slates, there and then not.",
        opts: &[
            opt(
                "Lie still",
                "It stops at last. You lie awake until grey light.",
                &[Vital(Nerve, -8), Vital(Rest, -12)],
            ),
            opt(
                "Bang on the ceiling",
                "Whatever it is goes, scraping. Everything for a mile heard you.",
                &[Noise(14), Vital(Nerve, 2)],
            ),
            Opt {
                label: "Go up with a light",
                req: &[Cond::Light],
                odds: Odds::Pct(55),
                ..Opt {
                    ok: Res {
                        text: "A heron, huge and furious, lifting off the ridge. You almost laugh, and then you do not.",
                        fx: &[Vital(Nerve, 6), Minutes(20)],
                    },
                    fail: Res {
                        text: "It is lying along the ridge, and its face turns to your light.",
                        fx: &[Fight(Creature::Runner, 1, Band::Near)],
                    },
                    ..BLANK
                }
            },
        ],
        ..E
    },
    EventDef {
        id: "dog_growls",
        ctx: Ctx::Night,
        weight: 18,
        req: &[Cond::Dog(DogIs::Companion)],
        once: false,
        title: "The dog growls",
        text: "The dog is up, stiff-legged at the window, growling under its breath at the dark.",
        opts: &[
            opt(
                "Hush it and lie still",
                "You hold its muzzle. After a while it lies down. Whatever it was has passed.",
                &[Vital(Nerve, -3)],
            ),
            opt(
                "Sit up with it",
                "You sit up by the window with the dog. Nothing comes, but you are ready if it does.",
                &[Vital(Rest, -12), Barricade(15), Flag("watching")],
            ),
            Opt {
                label: "Let it out",
                odds: Odds::Pct(65),
                ..Opt {
                    ok: Res {
                        text: "It goes out like a shot. A long time later it scratches to come back in, wet and pleased with itself.",
                        fx: &[Vital(Nerve, 4)],
                    },
                    fail: Res {
                        text: "It goes out like a shot. You hear it, far off, and then you do not.",
                        fx: &[Dog(DogFx::Lose), Vital(Nerve, -15)],
                    },
                    ..BLANK
                }
            },
        ],
        ..E
    },
    EventDef {
        id: "singing",
        ctx: Ctx::Night,
        weight: 10,
        req: &[Cond::MinDay(10)],
        title: "Singing",
        text: "From somewhere across the river, children singing. One note, held, without breath, without end.",
        opts: &[
            opt(
                "Cover your ears",
                "You lie with the pillow over your head until it stops, or until you sleep.",
                &[Vital(Nerve, -6), Vital(Rest, -8)],
            ),
            opt(
                "Go to the window",
                "You see them, on the far bank, small and pale, facing the mill. When you blink they are not there.",
                &[Vital(Nerve, -15), Codex("fg_small")],
            ),
            opt(
                "Sing over it",
                "You sing anything, loud, until your throat is raw. It works. It is also noise.",
                &[Noise(10), Vital(Nerve, 4)],
            ),
        ],
        ..E
    },
    EventDef {
        id: "radio_by_itself",
        ctx: Ctx::Night,
        weight: 8,
        req: &[Cond::Station(Station::Radio, 1), Cond::MinDay(8)],
        title: "The radio",
        text: "The wireless comes on by itself in the dark. The tone. Low and even, the one you know without having heard it.",
        opts: &[
            opt(
                "Pull the battery",
                "Silence. Your hands do not stop shaking for some time.",
                &[Vital(Nerve, -6)],
            ),
            opt(
                "Listen",
                "It goes on and on. After a while you find you are standing, facing the mill, and you do not remember standing.",
                &[Vital(Nerve, -18), Codex("radio_choir"), Noise(5)],
            ),
        ],
        ..E
    },
    // ---------------------------------------------------------------- Radio (while listening)
    EventDef {
        id: "radio_voice",
        ctx: Ctx::Radio,
        weight: 30,
        req: &[Cond::Station(Station::Radio, 2), Cond::MinDay(6)],
        title: "A voice asking",
        text: "A man's voice, close and clear: is anyone on the river. Say where you are. We can help. Say where you are.",
        opts: &[
            opt(
                "Tell him where you are",
                "You tell him: the lock house below the weir. He says, good. Stay there.",
                &[Flag("told_location"), Schedule("voice_again", 2)],
            ),
            opt(
                "Ask where he is",
                "He will not say. He asks again where you are, in exactly the same words.",
                &[Vital(Nerve, -6), Flag("voice_suspicious")],
            ),
            opt(
                "Switch it off",
                "The silence afterwards is very large.",
                &[Vital(Nerve, -2)],
            ),
        ],
        ..E
    },
    EventDef {
        id: "voice_again",
        ctx: Ctx::Radio,
        weight: 0,
        req: &[Cond::Flag("told_location")],
        title: "The voice again",
        text: "The same man. We saw the lock house, he says. We are coming up the towpath with a cart. Have food ready.",
        opts: &[
            opt(
                "Tell them to come",
                "He says tomorrow, maybe the day after.",
                &[Schedule("men_at_gate", 1)],
            ),
            opt(
                "Tell them to stay away",
                "A long pause. Then: suit yourself. The set hisses.",
                &[Vital(Nerve, -4)],
            ),
            opt(
                "Say nothing",
                "He says your name, which you never told him. Then nothing.",
                &[Vital(Nerve, -12), Schedule("men_at_gate", 2)],
            ),
        ],
        ..E
    },
    // ---------------------------------------------------------------- Road
    EventDef {
        id: "road_crowd",
        ctx: Ctx::Road,
        weight: 12,
        req: &[Cond::MinDay(6)],
        title: "A crowd in the lane",
        text: "The lane ahead is full of them. Thirty, forty, standing shoulder to shoulder, facing away from you.",
        opts: &[
            opt(
                "Go round through the fields",
                "Wet to the knees and slower, but none of them turn.",
                &[Minutes(40), Vital(Warmth, -6)],
            ),
            Opt {
                label: "Walk through them",
                odds: Odds::Skill(Skill::QuietStep, 45, 9),
                ..Opt {
                    ok: Res {
                        text: "You walk between them, close enough to touch. None of them turn.",
                        fx: &[Xp(Skill::QuietStep, 2), Vital(Nerve, -6)],
                    },
                    fail: Res {
                        text: "A heel scrapes. Two of them turn.",
                        fx: &[Fight(Creature::Stander, 2, Band::Near), Vital(Nerve, -8)],
                    },
                    ..BLANK
                }
            },
        ],
        ..E
    },
    EventDef {
        id: "road_cart",
        ctx: Ctx::Road,
        weight: 12,
        title: "An overturned cart",
        text: "A milk cart on its side, the horse dead in the traces. Crates spilled across the road.",
        opts: &[
            Opt {
                label: "Search it",
                minutes: 20,
                odds: Odds::Pct(75),
                ..Opt {
                    ok: Res {
                        text: "Under the crates: a few things worth carrying.",
                        fx: &[Loot(&[
                            ("tinned_peaches", 50, 1, 2),
                            ("rope", 60, 1, 1),
                            ("glass_bottle", 70, 1, 2),
                            ("leather", 50, 1, 1),
                        ])],
                    },
                    fail: Res {
                        text: "Something under the cart has hold of your ankle.",
                        fx: &[Fight(Creature::Stander, 1, Band::Close)],
                    },
                    ..BLANK
                }
            },
            opt("Walk on", "You leave it.", &[]),
        ],
        ..E
    },
    EventDef {
        id: "road_bell",
        ctx: Ctx::Road,
        weight: 10,
        req: &[Cond::MinDay(5)],
        title: "The bell",
        text: "The chapel bell rings once, far off, and the note holds. In the fields on either side, figures you had taken for posts turn their heads.",
        opts: &[
            opt(
                "Lie in the ditch until it fades",
                "You lie in cold water until the hum is gone from your teeth.",
                &[Minutes(30), Vital(Warmth, -10), Vital(Nerve, -4)],
            ),
            opt(
                "Keep walking",
                "You keep walking. You feel them watching for a long way.",
                &[Vital(Nerve, -8), Noise(6)],
            ),
        ],
        ..E
    },
    EventDef {
        id: "road_body",
        ctx: Ctx::Road,
        weight: 10,
        title: "A man who did not get up",
        text: "A man face down on the towpath, a pack still on his back. He fell when it came, and he has not moved since.",
        opts: &[
            opt(
                "Take the pack",
                "You roll him to get the straps. His eyes are open.",
                &[
                    Loot(&[
                        ("tinned_meat", 60, 1, 2),
                        ("bandage", 40, 1, 1),
                        ("lamp_oil", 30, 1, 1),
                        ("wire", 50, 1, 2),
                        ("satchel", 15, 1, 1),
                    ]),
                    Vital(Nerve, -5),
                ],
            ),
            opt(
                "Cover his face",
                "You put his cap over his face and go on. It helps, a little.",
                &[Vital(Nerve, 3)],
            ),
            opt("Walk on", "You step round him.", &[]),
        ],
        ..E
    },
    EventDef {
        id: "road_ice",
        ctx: Ctx::Road,
        weight: 14,
        req: &[Cond::MinDay(26)],
        title: "Ice on the towpath",
        text: "The towpath is glazed with ice from edge to edge, the river black beside it.",
        opts: &[
            opt("Go carefully", "Slow, and every step a decision.", &[Minutes(30)]),
            Opt {
                label: "Keep your pace",
                odds: Odds::Pct(55),
                ..Opt {
                    ok: Res {
                        text: "You keep your feet.",
                        fx: &[],
                    },
                    fail: Res {
                        text: "Your feet go and you come down hard on the edge of the path.",
                        fx: &[Vital(Health, -8), Bleed(1), Noise(4)],
                    },
                    ..BLANK
                }
            },
        ],
        ..E
    },
    EventDef {
        id: "fog_figure",
        ctx: Ctx::Road,
        weight: 18,
        req: &[Cond::Weather(Weather::Fog)],
        title: "A figure in the fog",
        text: "A figure in the fog ahead, walking toward you. Walking properly, heel and toe.",
        opts: &[
            Opt {
                label: "Call out",
                odds: Odds::Pct(55),
                ..Opt {
                    ok: Res {
                        text: "An old man with a stick. He tells you the estuary is open and the boats are still coming in, and walks on into the white.",
                        fx: &[Vital(Nerve, 10), Codex("radio_saltmouth")],
                    },
                    fail: Res {
                        text: "It stops when you call. Then it comes much faster.",
                        fx: &[Fight(Creature::Runner, 1, Band::Near), Noise(4)],
                    },
                    ..BLANK
                }
            },
            Opt {
                label: "Step off the path and wait",
                odds: Odds::Skill(Skill::QuietStep, 60, 7),
                ..Opt {
                    ok: Res {
                        text: "It passes an arm's length away and does not turn.",
                        fx: &[Xp(Skill::QuietStep, 1)],
                    },
                    fail: Res {
                        text: "It stops beside you.",
                        fx: &[Fight(Creature::Stander, 1, Band::Close)],
                    },
                    ..BLANK
                }
            },
        ],
        ..E
    },
    EventDef {
        id: "stranger_returns",
        ctx: Ctx::Road,
        weight: 0,
        req: &[Cond::Flag("took_cache")],
        title: "The stranger",
        text: "The man from the weir steps out of the hedge with the rifle level. You took from me, he says.",
        opts: &[
            Opt {
                label: "Give him something back",
                req: &[Cond::Meals(2)],
                ..then(
                    "You put food on the path. He takes it without lowering the rifle and backs away.",
                    &[Meals(2), Unflag("took_cache")],
                )
            },
            Opt {
                label: "Talk him down",
                odds: Odds::Nerve(45),
                ..Opt {
                    ok: Res {
                        text: "You talk. He listens. In the end he lowers the rifle and says keep it, and goes.",
                        fx: &[Unflag("took_cache"), Vital(Nerve, 5)],
                    },
                    fail: Res {
                        text: "He fires once, past your head, and runs. Every one of them in the valley heard it.",
                        fx: &[Noise(30), Vital(Nerve, -15), Unflag("took_cache")],
                    },
                    ..BLANK
                }
            },
            Opt {
                label: "Shoot first",
                req: &[Cond::Gun],
                ..then(
                    "It is over quickly. You stand on the path a long time afterwards.",
                    &[
                        Shot,
                        Noise(30),
                        Vital(Nerve, -25),
                        Unflag("took_cache"),
                        Loot(&[("rounds", 90, 2, 5), ("smoked_fish", 70, 1, 2)]),
                    ],
                )
            },
        ],
        ..E
    },
    // ---------------------------------------------------------------- Arrival
    EventDef {
        id: "stranger_weir",
        ctx: Ctx::Arrive,
        place: Some("weir"),
        weight: 30,
        req: &[Cond::MinDay(7)],
        title: "A man fishing",
        text: "A man on the weir steps, fishing, a rifle across his knees. He has seen you.",
        opts: &[
            Opt {
                label: "Approach with open hands",
                odds: Odds::Nerve(55),
                ..Opt {
                    ok: Res {
                        text: "He lets you sit. He gives you a fish, tells you the coal wharf has fuel, and goes before noon.",
                        fx: &[Item("raw_fish", 2), Reveal("wharf"), Vital(Nerve, 8)],
                    },
                    fail: Res {
                        text: "The rifle comes round. You back away up the steps.",
                        fx: &[Vital(Nerve, -10)],
                    },
                    ..BLANK
                }
            },
            opt(
                "Watch from the reeds",
                "He fishes, eats, and hides something under the keeper's hut before he goes.",
                &[Minutes(40), Flag("saw_cache"), Schedule("stranger_cache", 0)],
            ),
            opt("Leave him be", "You keep your distance.", &[]),
        ],
        ..E
    },
    EventDef {
        id: "stranger_cache",
        ctx: Ctx::Arrive,
        place: Some("weir"),
        weight: 0,
        req: &[Cond::Flag("saw_cache")],
        title: "The stranger's cache",
        text: "Under the keeper's hut, a tin box wrapped in oilcloth. His.",
        opts: &[
            opt(
                "Take it",
                "Rounds, smoked fish, a pencil map.",
                &[
                    Item("rounds", 4),
                    Item("smoked_fish", 2),
                    Codex("note_stranger"),
                    Flag("took_cache"),
                    Unflag("saw_cache"),
                    Schedule("stranger_returns", 3),
                ],
            ),
            opt("Leave it", "You leave it where it is.", &[Unflag("saw_cache")]),
        ],
        ..E
    },
    EventDef {
        id: "orme_bridge",
        ctx: Ctx::Arrive,
        place: Some("mill"),
        weight: 40,
        req: &[Cond::Codex("note_orme")],
        title: "Mr Orme",
        text: "On the mill bridge, a man in a lock-keeper's waistcoat stands at the rail, looking down at the water. You know the coat. You know the set of the shoulders.",
        opts: &[
            opt(
                "Go to him",
                "His face is calm. His eyes are open. You take the keys from his pocket because he would want you to, and you leave him looking at the river.",
                &[
                    Flag("met_orme"),
                    Item("key_mill", 1),
                    Vital(Nerve, -10),
                    Codex("fg_still"),
                ],
            ),
            Opt {
                label: "Put him down",
                req: &[Cond::Weapon],
                ..then(
                    "It takes one blow. He does not turn. You take his keys after.",
                    &[Flag("met_orme"), Item("key_mill", 1), Vital(Nerve, -18), Kill],
                )
            },
            opt(
                "Leave him be",
                "You cross by the other bridge.",
                &[Flag("met_orme"), Vital(Nerve, -4), Minutes(15)],
            ),
        ],
        ..E
    },
    EventDef {
        id: "mill_note",
        ctx: Ctx::Arrive,
        place: Some("mill"),
        weight: 20,
        req: &[Cond::MinDay(12)],
        title: "The note",
        text: "A sound from inside the mill, so low you feel it in your knees before you hear it. Every window in the place hums with it.",
        opts: &[
            opt(
                "Stop your ears and go in",
                "Cloth in your ears and your hands over them. It is still in your teeth.",
                &[Vital(Nerve, -12), Codex("fg_sound")],
            ),
            opt(
                "Wait for it to stop",
                "It stops after an hour. You do not know why.",
                &[Minutes(60), Vital(Nerve, -4)],
            ),
        ],
        ..E
    },
    EventDef {
        id: "light_in_window",
        ctx: Ctx::Arrive,
        weight: 14,
        req: &[Cond::Dark],
        title: "A light",
        text: "A light in an upper window. It does not move.",
        opts: &[
            opt(
                "Watch it",
                "It burns down and goes out. A lamp left burning since the night it happened. There may be oil left.",
                &[Minutes(30), Loot(&[("lamp_oil", 80, 1, 1)])],
            ),
            opt("Go in anyway", "You go in under it.", &[Vital(Nerve, -4)]),
        ],
        ..E
    },
    EventDef {
        id: "widow_ferry",
        ctx: Ctx::Arrive,
        place: Some("weir"),
        weight: 30,
        req: &[Cond::Trades(2), Cond::Flag("widow_here")],
        title: "The widow's question",
        text: "The widow writes on her slate and turns it to you: Have you been to the ferry.",
        opts: &[
            opt(
                "Tell her you mean to take it",
                "She looks at you a long time. Then she writes: Pell has the coil. And under it: He was my husband's friend.",
                &[Reveal("garage"), Flag("widow_knows")],
            ),
            opt(
                "Ask about her husband",
                "She writes: Tom. Then rubs it out. Then writes: I could not hear it. I have not heard anything in twenty years.",
                &[Codex("photo_wedding"), Vital(Nerve, 3)],
            ),
            opt("Say nothing", "She shrugs and wipes the slate.", &[]),
        ],
        ..E
    },
    EventDef {
        id: "widow_photo",
        ctx: Ctx::Arrive,
        place: Some("weir"),
        weight: 40,
        req: &[
            Cond::Flag("widow_knows"),
            Cond::Codex("photo_launch"),
            Cond::Flag("widow_here"),
        ],
        title: "The launch photograph",
        text: "You have the photograph of the Kittiwake's launch. The woman on the bank, hands over her ears, smiling, is her.",
        opts: &[
            opt(
                "Give it to her",
                "She holds it a long time. Then she writes: Fuel. I have one drum. It is yours.",
                &[WidowStock("fuel_drum"), Vital(Nerve, 8)],
            ),
            opt("Keep it", "You keep it.", &[]),
        ],
        ..E
    },
    // ---------------------------------------------------------------- Rooms
    EventDef {
        id: "child_on_stairs",
        ctx: Ctx::Room,
        place: Some("school"),
        room: Some("school_stairs"),
        weight: 100,
        title: "On the stairs",
        text: "A child sits on the third stair, facing the wall, a satchel in her lap. Her lips are moving.",
        opts: &[
            Opt {
                label: "Step past her",
                odds: Odds::Skill(Skill::QuietStep, 55, 8),
                ..Opt {
                    ok: Res {
                        text: "You go up past her with your back to the other wall. She does not turn.",
                        fx: &[Xp(Skill::QuietStep, 2), Vital(Nerve, -6)],
                    },
                    fail: Res {
                        text: "A stair cracks under you. She turns.",
                        fx: &[Fight(Creature::Small, 1, Band::Close), Vital(Nerve, -10)],
                    },
                    ..BLANK
                }
            },
            Opt {
                label: "Carry her out to the yard",
                odds: Odds::Pct(70),
                ..Opt {
                    ok: Res {
                        text: "She weighs nothing. You set her down in the yard, facing the gate, and go back in.",
                        fx: &[Vital(Nerve, -14), Flag("carried_child"), Codex("fg_still")],
                    },
                    fail: Res {
                        text: "Halfway down, she starts to hum.",
                        fx: &[Fight(Creature::Small, 1, Band::Close), Vital(Nerve, -16)],
                    },
                    ..BLANK
                }
            },
            Opt {
                label: "Put her down",
                req: &[Cond::Weapon],
                ..then(
                    "You do it quickly. You will think about it later, a great deal.",
                    &[Vital(Nerve, -22), Noise(3), Kill],
                )
            },
        ],
        ..E
    },
    EventDef {
        id: "hale_patient",
        ctx: Ctx::Room,
        place: Some("surgery"),
        room: Some("surg_consult"),
        weight: 100,
        title: "The patient",
        text: "A man strapped to the examination couch. One of them. Someone strapped him down carefully and left a note on his chest.",
        opts: &[
            opt(
                "Read the note",
                "It says: Mr Kell. Heard it first, a month ago. Do not unstrap. R.H.",
                &[Vital(Nerve, -5), Codex("note_hale_cases")],
            ),
            Opt {
                label: "Put him down",
                req: &[Cond::Weapon],
                ..then(
                    "He does not struggle against the straps.",
                    &[Vital(Nerve, -10), Kill, Noise(2)],
                )
            },
            opt(
                "Leave him",
                "You work around him. His eyes follow nothing.",
                &[Vital(Nerve, -3)],
            ),
        ],
        ..E
    },
    EventDef {
        id: "cells_knock",
        ctx: Ctx::Room,
        place: Some("station"),
        room: Some("pol_cells"),
        weight: 100,
        title: "The knocking",
        text: "The knocking comes from the last cell. Slow, the same weight each time. The key is in the door.",
        opts: &[
            Opt {
                label: "Open it",
                odds: Odds::Pct(40),
                ..Opt {
                    ok: Res {
                        text: "A man lies on the bunk, long dead, alive when they locked him in. It is the water pipe knocking. His notebook is under the pillow.",
                        fx: &[
                            Vital(Nerve, -6),
                            Loot(&[("tobacco", 80, 1, 1), ("lockpicks", 60, 1, 1)]),
                        ],
                    },
                    fail: Res {
                        text: "It has been knocking with its forehead.",
                        fx: &[Fight(Creature::Runner, 1, Band::Close)],
                    },
                    ..BLANK
                }
            },
            opt(
                "Leave it locked",
                "The knocking follows you down the corridor.",
                &[Vital(Nerve, -4)],
            ),
        ],
        ..E
    },
    EventDef {
        id: "child_in_yard",
        ctx: Ctx::Arrive,
        place: Some("school"),
        weight: 60,
        req: &[Cond::Flag("carried_child")],
        title: "In the yard",
        text: "The child you carried out is still in the yard. She has turned round. She is facing the mill.",
        opts: &[
            opt(
                "Go past",
                "You go in the other way.",
                &[Vital(Nerve, -6), Codex("fg_sound")],
            ),
            opt(
                "Turn her back to the gate",
                "You turn her by the shoulders. She is very cold.",
                &[Vital(Nerve, -10)],
            ),
        ],
        ..E
    },
];

pub fn get(id: &str) -> Option<&'static EventDef> {
    EVENTS.iter().find(|e| e.id == id)
}

pub fn check_integrity() {
    use super::{codex, items, places};
    let mut chained = 0;
    for e in EVENTS {
        assert!(
            !e.opts.is_empty() && e.opts.len() <= 4,
            "{} has {} options",
            e.id,
            e.opts.len()
        );
        assert!(!e.text.contains('!'), "exclamation in {}", e.id);
        if let Some(p) = e.place {
            let pd = places::get(p).unwrap_or_else(|| panic!("{} at unknown place {p}", e.id));
            if let Some(r) = e.room {
                assert!(pd.rooms.iter().any(|x| x.id == r), "{} at unknown room {r}", e.id);
            }
        }
        if e.weight == 0 {
            chained += 1;
        }
        let conds = e.req.iter().chain(e.opts.iter().flat_map(|o| o.req.iter()));
        for c in conds {
            match c {
                Cond::Item(id, _) => assert!(items::get(id).is_some(), "{}: {id}", e.id),
                Cond::Codex(id) => assert!(codex::get(id).is_some(), "{}: {id}", e.id),
                _ => {}
            }
        }
        for o in e.opts {
            assert!(!o.label.is_empty() && !o.ok.text.is_empty(), "{}: empty option", e.id);
            assert!(!o.ok.text.contains('!') && !o.fail.text.contains('!'));
            let has_fail = !matches!(o.odds, Odds::Always);
            assert_eq!(has_fail, !o.fail.text.is_empty(), "{}: {} fail text", e.id, o.label);
            for fx in o.ok.fx.iter().chain(o.fail.fx) {
                match fx {
                    Fx::Item(id, _) | Fx::WidowStock(id) => assert!(items::get(id).is_some(), "{}: {id}", e.id),
                    Fx::Schedule(id, _) => assert!(get(id).is_some(), "{}: schedules {id}", e.id),
                    Fx::Reveal(id) => assert!(places::get(id).is_some(), "{}: {id}", e.id),
                    Fx::Codex(id) => assert!(codex::get(id).is_some(), "{}: {id}", e.id),
                    Fx::Loot(l) => {
                        for (id, _, _, _) in l.iter() {
                            assert!(items::get(id).is_some(), "{}: {id}", e.id);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    assert!(EVENTS.len() >= 30, "only {} events", EVENTS.len());
    assert!(chained >= 6);
}
